/// # Small Core
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SmallCore {}

/// # Core
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    pub const fn small_core_count(&self) -> usize {
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Sha<const C: usize, const SC: usize, const CSC: usize, const D: usize> {
    cores: [Core<CSC>; C],
}

impl<const C: usize, const SC: usize, const CSC: usize, const D: usize> Sha<C, SC, CSC, D> {
    pub fn new() -> Self {
        Sha {
            cores: [Core::<CSC>::new(); C],
        }
    }

    /// ## Get the number of Cores in the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Sha;
    ///
    /// let asic = Sha::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.core_count(), 168);
    /// ```
    pub const fn core_count(&self) -> usize {
        C
    }

    /// ## Get the number of Small Cores in the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Sha;
    ///
    /// let asic = Sha::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.small_core_count(), 672);
    /// ```
    pub const fn small_core_count(&self) -> usize {
        SC
    }

    /// ## Get the number of Small Cores in a single Core of the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Sha;
    ///
    /// let asic = Sha::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.core_small_core_count(), 4);
    /// ```
    pub const fn core_small_core_count(&self) -> usize {
        CSC
    }

    /// ## Get the number of Domains in the ASIC
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::sha::Sha;
    ///
    /// let asic = Sha::<168, 672, 4, 4>::new(); // BM1397
    /// assert_eq!(asic.domain_count(), 4);
    /// ```
    pub const fn domain_count(&self) -> usize {
        D
    }
}

impl<const C: usize, const SC: usize, const CSC: usize, const D: usize> Default
    for Sha<C, SC, CSC, D>
{
    fn default() -> Self {
        Self::new()
    }
}
