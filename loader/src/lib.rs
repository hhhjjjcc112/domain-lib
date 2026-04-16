#![no_std]

mod vm;

extern crate alloc;
#[macro_use]
extern crate log;
use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{
    fmt::{Debug, Formatter},
    ops::Range,
};

use log::{debug, trace};
use memory_addr::VirtAddr;
use storage::StorageArg;
pub use vm::{DomainArea, DomainVmOps};
use xmas_elf::{program::Type, sections::SectionData, symbol_table::Entry, ElfFile};

use crate::vm::DomainMappingFlags;
const FRAME_SIZE: usize = 4096;
type Result<T> = core::result::Result<T, &'static str>;

pub struct DomainLoader<V: DomainVmOps> {
    entry_point: usize,
    data: Arc<Vec<u8>>,
    virt_start: usize,
    module_area: Option<Box<dyn DomainArea>>,
    ident: String,
    text_section: Range<usize>,
    _phantom: core::marker::PhantomData<V>,
}

impl<V: DomainVmOps> Debug for DomainLoader<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DomainLoader")
            .field("entry", &self.entry_point)
            .field("phy_start", &self.virt_start)
            .field("ident", &self.ident)
            .field("text_section", &self.text_section)
            .finish()
    }
}

impl<V: DomainVmOps> Clone for DomainLoader<V> {
    fn clone(&self) -> Self {
        Self {
            entry_point: 0,
            data: self.data.clone(),
            virt_start: 0,
            ident: self.ident.to_string(),
            module_area: None,
            text_section: self.text_section.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<V: DomainVmOps> DomainLoader<V> {
    pub fn new(data: Arc<Vec<u8>>, ident: &str) -> Self {
        Self {
            entry_point: 0,
            data,
            virt_start: 0,
            ident: ident.to_string(),
            module_area: None,
            text_section: 0..0,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Return the domain file info(name, size)
    pub fn domain_file_info(&self) -> (String, usize) {
        (self.ident.clone(), self.data.len())
    }

    pub fn empty() -> Self {
        Self::new(Arc::new(vec![]), "empty_loader")
    }

    fn entry_point(&self) -> usize {
        self.entry_point
    }

    pub fn virt_start(&self) -> usize {
        self.virt_start
    }

    pub fn entry_addr(&self) -> usize {
        self.entry_point
    }

    pub fn call<T: ?Sized, C: 'static + ?Sized, F>(
        &self,
        id: u64,
        use_old_id: Option<u64>,
        callback: F,
    ) -> Box<T>
    where
        F: FnOnce(
            Option<u64>,
        ) -> (
            &'static C,
            &'static dyn shared_heap::SharedHeapAlloc,
            StorageArg,
        ),
    {
        type F<T, C> =
            fn(&'static C, u64, &'static dyn shared_heap::SharedHeapAlloc, StorageArg) -> Box<T>;
        let main =
            unsafe { core::mem::transmute::<*const (), F<T, C>>(self.entry_point() as *const ()) };
        let (syscall, heap, storage_arg) = callback(use_old_id);
        main(syscall, id, heap, storage_arg)
    }

    fn load_program(&mut self, elf: &ElfFile) -> Result<()> {
        // 按 ELF 的 PT_LOAD 段逐个复制到已经分配好的连续运行时映像中。
        //
        // 这里的核心规则是：ELF 里记录的是“链接视角下的虚拟地址”，而 loader
        // 给域分配的是“运行时的实际映像基址 virt_start”。因此每个段的拷贝位置都
        // 必须先做一次平移：runtime_addr = virt_start + elf_vaddr。
        //
        // 这一层平移不会改变 ELF 内部的相对布局，只是把整个镜像从链接地址搬到
        // 当前进程里刚分配出来的那片连续内存中。
        elf.program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .for_each(|ph| {
                // 段起始/结束地址都要按运行时基址平移。
                let start_vaddr = ph.virtual_addr() as usize + self.virt_start;
                let end_vaddr = start_vaddr + ph.mem_size() as usize;
                let mut permission = DomainMappingFlags::empty();
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    permission |= DomainMappingFlags::READ;
                }
                if ph_flags.is_write() {
                    permission |= DomainMappingFlags::WRITE;
                }
                if ph_flags.is_execute() {
                    permission |= DomainMappingFlags::EXECUTE;
                }
                let vaddr = VirtAddr::from(start_vaddr).align_down_4k().as_usize();
                let end_vaddr = VirtAddr::from(end_vaddr).align_up_4k().as_usize();
                // 这里保留页对齐后的 text 段范围，后面会用它统一切回可执行权限。
                let data =
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
                let data_len = data.len();
                // 段数据只覆盖 file_size 的部分；mem_size 里超出的区域由前面的整体清零
                // 保证成 bss / 段间空洞。
                let module_area = self.module_area.as_ref().unwrap();
                let module_slice = module_area.as_mut_slice();
                let copy_start = start_vaddr - self.virt_start;
                // copy_start 仍然是运行时映像内的相对偏移，不是原始 ELF 偏移。
                module_slice[copy_start..copy_start + data_len].copy_from_slice(data);
                if permission.contains(DomainMappingFlags::EXECUTE) {
                    // 只有真正可执行的段才会进入 text_section，后面统一改回 RX。
                    self.text_section = vaddr..end_vaddr;
                }
            });
        Ok(())
    }
    fn relocate_dyn(&self, elf: &ElfFile) -> Result<()> {
        // load() 已经把原始段内容放进运行时映像，这里再补上动态重定位，
        // 把链接期留下的占位地址改成运行时真实地址。
        let res = relocate_dyn(elf, self.virt_start)?;
        trace!("Relocate_dyn {} entries", res.len());
        res.into_iter().for_each(|kv| {
            // kv.0 是要写回的内存地址，kv.1 是最终修正后的目标地址。
            trace!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
            let addr = kv.0;
            unsafe { (addr as *mut usize).write(kv.1) }
        });
        trace!("Relocate_dyn done");
        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        let data = self.data.clone();
        let elf_binary = data.as_slice();
        const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
        if elf_binary[0..4] != ELF_MAGIC {
            return Err("not a elf file");
        }
        debug!("Domain address:{:p}", elf_binary.as_ptr());
        let elf = ElfFile::new(elf_binary)?;
        debug!("Domain type:{:?}", elf.header.pt2.type_().as_type());
        // 用最后一个 PT_LOAD 的末尾地址估算整个镜像的覆盖范围，
        // 这样才能一次性分配出足够大的连续运行时区域。
        let end_paddr = elf
            .program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .last()
            .map(|x| x.virtual_addr() as usize + x.mem_size() as usize)
            .unwrap();
        let end_paddr = VirtAddr::from(end_paddr).align_up(FRAME_SIZE);
        // 分配域内连续映像；loader 后续所有段拷贝和重定位都在这块区域里完成。
        let module_area = V::map_domain_area(end_paddr.as_usize());
        let region_start = module_area.start_virtual_address().as_usize();
        // 先清零整段映像，原因有两个：
        // 1) .bss 语义要求未初始化部分必须是 0；
        // 2) ELF 段之间如果存在空洞，也应当按零填充来表现。
        module_area.as_mut_slice().fill(0);
        self.virt_start = region_start;
        self.module_area = Some(module_area);
        // 先拷贝段内容，再做重定位。顺序不能反过来：重定位写入的目标地址
        // 依赖已经放好的段数据和已经确定的运行时基址。
        self.load_program(&elf)?;
        self.relocate_dyn(&elf)?;
        // 重定位完成后，再把 text 段切回可执行权限。
        if self.text_section.end > self.text_section.start {
            let text_pages = (self.text_section.end - self.text_section.start) / FRAME_SIZE;
            V::set_memory_x(self.text_section.start, text_pages)?;
        }
        // 最终入口 = ELF 文件记录的入口点 + 运行时装载基址。
        let entry = elf.header.pt2.entry_point() as usize + region_start;
        self.entry_point = entry;
        Ok(())
    }
}

impl<V: DomainVmOps> Drop for DomainLoader<V> {
    fn drop(&mut self) {
        info!("drop domain loader [{}]", self.ident);
        if let Some(module_area) = self.module_area.take() {
            V::unmap_domain_area(module_area)
        }
    }
}

fn relocate_dyn(elf: &ElfFile, region_start: usize) -> Result<Vec<(usize, usize)>> {
    // 这里做的是“装载重定向”：把 ELF 里在链接期留下的重定位项，统一改写成
    // 当前运行时基址 region_start 下的真实地址。
    //
    // 之所以要单独封装成 relocate_dyn，是因为不同段里可能混有 .rela.dyn 和
    // .rela.plt，两者本质上都是“告诉 loader 哪些位置需要被改写”，只是来源不同。
    let mut res = vec![];
    apply_relocation_section(elf, ".rela.dyn", region_start, &mut res)?;
    apply_relocation_section(elf, ".rela.plt", region_start, &mut res)?;
    Ok(res)
}

fn apply_relocation_section(
    elf: &ElfFile,
    section_name: &str,
    region_start: usize,
    res: &mut Vec<(usize, usize)>,
) -> Result<()> {
    // 有些 ELF 不一定带这个段；缺失时直接跳过即可。
    let Some(section) = elf.find_section_by_name(section_name) else {
        return Ok(());
    };
    // 重定位表必须是 Rela64，里面同时记录了 offset 和 addend。
    let data = section
        .get_data(elf)
        .map_err(|_| "bad relocation section")?;
    let entries = match data {
        SectionData::Rela64(entries) => entries,
        _ => return Err("bad relocation section"),
    };
    // 对于符号型重定位，需要从 .dynsym 里查到目标符号的值。
    // 如果当前镜像完全没有 .dynsym，那么只能处理纯相对重定位。
    let dynsym = match elf.find_section_by_name(".dynsym") {
        Some(section) => Some(match section.get_data(elf).map_err(|_| "bad .dynsym")? {
            SectionData::DynSymbolTable64(entries) => entries,
            _ => return Err("bad .dynsym"),
        }),
        None => None,
    };

    for entry in entries.iter() {
        // offset 是“需要被写回的位置”相对于镜像基址的偏移；
        // addend 是链接器在 relocation 里预先放好的附加项。
        let offset = entry.get_offset() as usize;
        let addend = entry.get_addend() as usize;
        // 这里才是真正写回的运行时地址。
        let addr = region_start + offset;
        let ty = entry.get_type();
        match ty {
            RELATIVE => {
                // RELATIVE 是最常见、也最适合 PIE 域镜像的重定位类型。
                // 它不依赖外部符号解析，只需要把“基址 + addend”写回目标地址即可。
                // 这也是当前 loader 能够支持大多数域镜像的关键原因。
                res.push((addr, region_start + addend));
            }
            ABSOLUTE | GLOB_DAT | JUMP_SLOT => {
                // 这类重定位依赖符号表：需要先找到符号本身，再结合运行时基址修正。
                let dynsym = dynsym.ok_or("missing .dynsym for symbolic relocation")?;
                let sym = dynsym
                    .get(entry.get_symbol_table_index() as usize)
                    .ok_or("bad relocation symbol index")?;
                let sym_name = sym.get_name(elf).unwrap_or("<invalid>");
                if sym.shndx() == 0 {
                    // shndx == 0 表示符号未定义，说明当前镜像里找不到该符号的实现。
                    // 继续运行只会留下悬空地址，因此这里必须失败，让问题尽早暴露。
                    error!(
                        "[loader] unresolved {} in {}: symbol={}, offset={:#x}",
                        relocation_name(ty),
                        section_name,
                        sym_name,
                        offset
                    );
                    return Err("unresolved relocation symbol");
                }
                // 符号的最终地址 = 运行时基址 + 符号在镜像内的值 + addend。
                // 这里的 sym.value() 仍然是链接期意义上的镜像内偏移，所以同样需要
                // 按 region_start 重新平移。
                let value = region_start + sym.value() as usize + addend;
                res.push((addr, value));
            }
            t => {
                // 当前 loader 只实现当前最小可运行链路所需的重定位类型。
                // 一旦镜像里出现其它类型，说明构建产物已经超出了现有 loader 语义，
                // 继续静默忽略会让后续错误更难排查。
                error!(
                    "[loader] unsupported {}({}) in {}: offset={:#x}, addend={:#x}",
                    relocation_name(t),
                    t,
                    section_name,
                    offset,
                    addend
                );
                return Err("unsupported relocation type");
            }
        }
    }
    Ok(())
}

#[cfg(target_arch = "riscv64")]
const R_RISCV_64: u32 = 2;
#[cfg(target_arch = "riscv64")]
const R_RISCV_RELATIVE: u32 = 3;
#[cfg(target_arch = "riscv64")]
const R_RISCV_COPY: u32 = 4;
#[cfg(target_arch = "riscv64")]
const R_RISCV_JUMP_SLOT: u32 = 5;
#[cfg(target_arch = "x86_64")]
const R_X86_64_64: u32 = 1;
#[cfg(target_arch = "x86_64")]
const R_X86_64_COPY: u32 = 5;
#[cfg(target_arch = "x86_64")]
const R_X86_64_GLOB_DAT: u32 = 6;
#[cfg(target_arch = "x86_64")]
const R_X86_64_JUMP_SLOT: u32 = 7;
#[cfg(target_arch = "x86_64")]
const R_X86_64_RELATIVE: u32 = 8;
#[cfg(target_arch = "x86_64")]
const R_X86_64_IRELATIVE: u32 = 37;

#[cfg(target_arch = "riscv64")]
const RELATIVE: u32 = R_RISCV_RELATIVE;
#[cfg(target_arch = "riscv64")]
const ABSOLUTE: u32 = R_RISCV_64;
#[cfg(target_arch = "riscv64")]
const GLOB_DAT: u32 = u32::MAX;
#[cfg(target_arch = "riscv64")]
const JUMP_SLOT: u32 = R_RISCV_JUMP_SLOT;

#[cfg(target_arch = "x86_64")]
const RELATIVE: u32 = R_X86_64_RELATIVE;
#[cfg(target_arch = "x86_64")]
const ABSOLUTE: u32 = R_X86_64_64;
#[cfg(target_arch = "x86_64")]
const GLOB_DAT: u32 = R_X86_64_GLOB_DAT;
#[cfg(target_arch = "x86_64")]
const JUMP_SLOT: u32 = R_X86_64_JUMP_SLOT;

#[cfg(target_arch = "riscv64")]
fn relocation_name(ty: u32) -> &'static str {
    match ty {
        R_RISCV_64 => "R_RISCV_64",
        R_RISCV_RELATIVE => "R_RISCV_RELATIVE",
        R_RISCV_COPY => "R_RISCV_COPY",
        R_RISCV_JUMP_SLOT => "R_RISCV_JUMP_SLOT",
        _ => "UNKNOWN",
    }
}

#[cfg(target_arch = "x86_64")]
fn relocation_name(ty: u32) -> &'static str {
    match ty {
        R_X86_64_64 => "R_X86_64_64",
        R_X86_64_COPY => "R_X86_64_COPY",
        R_X86_64_GLOB_DAT => "R_X86_64_GLOB_DAT",
        R_X86_64_JUMP_SLOT => "R_X86_64_JUMP_SLOT",
        R_X86_64_RELATIVE => "R_X86_64_RELATIVE",
        R_X86_64_IRELATIVE => "R_X86_64_IRELATIVE",
        _ => "UNKNOWN",
    }
}
