#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum SourceLibrary {
    #[default]
    BestSource,
    FFMS2,
    LSMASH,
}

impl SourceLibrary {
    pub fn as_str(&self) -> &str {
        match self {
            SourceLibrary::BestSource => "BestSource",
            SourceLibrary::FFMS2 => "FFMS2",
            SourceLibrary::LSMASH => "L-SMASH",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PixelFormat {
    Yuv420p,
    Yuv420p10le,
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::Yuv420p10le
    }
}

impl PixelFormat {
    pub fn as_str(&self) -> &str {
        match self {
            PixelFormat::Yuv420p => "yuv420p",
            PixelFormat::Yuv420p10le => "yuv420p10le",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ColorPrimaries {
    Bt709,       // [1] BT.709
    Unspecified, // [2] unspecified, default
    Bt470m,      // [4] BT.470 System M (historical)
    Bt470bg,     // [5] BT.470 System B, G (historical)
    Bt601,       // [6] BT.601
    Smpte240,    // [7] SMPTE 240
    Film,        // [8] Generic film (color filters using illuminant C)
    Bt2020,      // [9] SMPTE 428 (CIE 1921 XYZ)
    Xyz,         // [10] SMPTE RP 431-2
    Smpte431,    // [11] SMPTE EG 431-2
    Smpte432,    // [12] SMPTE EG 432-1
    Ebu3213,     // [22] EBU Tech. 3213-E
}

impl Default for ColorPrimaries {
    fn default() -> Self {
        ColorPrimaries::Unspecified
    }
}

impl ColorPrimaries {
    pub fn as_str(&self) -> &str {
        match self {
            ColorPrimaries::Bt709 => "1",
            ColorPrimaries::Unspecified => "2",
            ColorPrimaries::Bt470m => "4",
            ColorPrimaries::Bt470bg => "5",
            ColorPrimaries::Bt601 => "6",
            ColorPrimaries::Smpte240 => "7",
            ColorPrimaries::Film => "8",
            ColorPrimaries::Bt2020 => "9",
            ColorPrimaries::Xyz => "10",
            ColorPrimaries::Smpte431 => "11",
            ColorPrimaries::Smpte432 => "12",
            ColorPrimaries::Ebu3213 => "22",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MatrixCoefficients {
    Identity,    // [0] Identity matrix
    Bt709,       // [1] BT.709
    Unspecified, // [2] unspecified, default
    Fcc,         // [4] US FCC 73.628
    Bt470bg,     // [5] BT.470 System B, G (historical)
    Bt601,       // [6] BT.601
    Smpte240,    // [7] SMPTE 240 M
    Ycgco,       // [8] YCgCo
    Bt2020Ncl,   // [9] BT.2020 non-constant luminance, BT.2100 YCbCr
    Bt2020Cl,    // [10] BT.2020 constant luminance
    Smpte2085,   // [11] SMPTE ST 2085 YDzDx
    ChromaNcl,   // [12] Chromaticity-derived non-constant luminance
    ChromaCl,    // [13] Chromaticity-derived constant luminance
    Ictcp,       // [14] BT.2100 ICtCp
}

impl Default for MatrixCoefficients {
    fn default() -> Self {
        MatrixCoefficients::Unspecified
    }
}

impl MatrixCoefficients {
    pub fn as_str(&self) -> &str {
        match self {
            MatrixCoefficients::Identity => "0",
            MatrixCoefficients::Bt709 => "1",
            MatrixCoefficients::Unspecified => "2",
            MatrixCoefficients::Fcc => "4",
            MatrixCoefficients::Bt470bg => "5",
            MatrixCoefficients::Bt601 => "6",
            MatrixCoefficients::Smpte240 => "7",
            MatrixCoefficients::Ycgco => "8",
            MatrixCoefficients::Bt2020Ncl => "9",
            MatrixCoefficients::Bt2020Cl => "10",
            MatrixCoefficients::Smpte2085 => "11",
            MatrixCoefficients::ChromaNcl => "12",
            MatrixCoefficients::ChromaCl => "13",
            MatrixCoefficients::Ictcp => "14",
        }
    }
}
