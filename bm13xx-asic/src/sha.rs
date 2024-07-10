/// # Small Core
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SmallCore {}

/// # Core
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Core<const SC: usize> {
    small_cores: [SmallCore; SC],
}

impl<const SC: usize> Core<SC> {
    pub fn new() -> Self {
        Core {
            small_cores: [SmallCore::default(); SC],
        }
    }

    /// ## Get the number of Small Cores in the Core
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Core;
    ///
    /// let core = Core::<4>::new();
    /// assert_eq!(core.small_core_count(), 4);
    /// ```
    pub fn small_core_count(&self) -> usize {
        SC
    }
}

impl<const SC: usize> Default for Core<SC> {
    fn default() -> Self {
        Self::new()
    }
}

/// # ASIC
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Asic<const C: usize, const SC: usize, const CSC: usize, const D: usize> {
    cores: [Core<CSC>; C],
    small_cores_cnt: usize,
    domain_cnt: usize,
}

impl<const C: usize, const SC: usize, const CSC: usize, const D: usize> Asic<C, SC, CSC, D> {
    pub fn new() -> Self {
        Asic {
            cores: [Core::<CSC>::new(); C],
            small_cores_cnt: SC,
            domain_cnt: D,
        }
    }

    /// ## Get the number of Cores in the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Asic;
    ///
    /// let asic = Asic::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.core_count(), 168);
    /// ```
    pub fn core_count(&self) -> usize {
        C
    }

    /// ## Get the number of Small Cores in the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Asic;
    ///
    /// let asic = Asic::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.small_core_count(), 672);
    /// ```
    pub fn small_core_count(&self) -> usize {
        self.small_cores_cnt
    }

    /// ## Get the number of Domains in the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Asic;
    ///
    /// let asic = Asic::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.domain_count(), 4);
    /// ```
    pub fn domain_count(&self) -> usize {
        self.domain_cnt
    }
}

impl<const C: usize, const SC: usize, const CSC: usize, const D: usize> Default
    for Asic<C, SC, CSC, D>
{
    fn default() -> Self {
        Self::new()
    }
}
