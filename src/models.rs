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

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TransferCharacteristics {
    Bt709,        // [1] BT.709
    Unpsecified,  // [2] unspecified, default
    Bt470m,       // [4] BT.470 System M (historical)
    Bt470bg,      // [5] BT.470 System B, G (historical)
    Bt601,        // [6] BT.601
    Smpte240,     // [7] SMPTE 240 M
    Linear,       // [8] Linear
    Log100,       // [9] Logarithmic (100 : 1 range)
    Log100Sqrt10, // [10] Logarithmic (100 * Sqrt(10) : 1 range)
    Iec61966,     // [11] IEC 61966-2-4
    Bt1361,       // [12] BT.1361
    Srgb,         // [13] sRGB or sYCC
    Bt202010,     // [14] BT.2020 10-bit systems
    Bt202012,     // [15] BT.2020 12-bit systems
    Smpte2084,    // [16] SMPTE ST 2084, ITU BT.2100 PQ
    Smpte428,     // [17] SMPTE ST 428
    Hlg,          // [18] BT.2100 HLG, ARIB STD-B67
}

impl Default for TransferCharacteristics {
    fn default() -> Self {
        TransferCharacteristics::Unpsecified
    }
}

impl TransferCharacteristics {
    pub fn as_str(&self) -> &str {
        match self {
            TransferCharacteristics::Bt709 => "1",
            TransferCharacteristics::Unpsecified => "2",
            TransferCharacteristics::Bt470m => "4",
            TransferCharacteristics::Bt470bg => "5",
            TransferCharacteristics::Bt601 => "6",
            TransferCharacteristics::Smpte240 => "7",
            TransferCharacteristics::Linear => "8",
            TransferCharacteristics::Log100 => "9",
            TransferCharacteristics::Log100Sqrt10 => "10",
            TransferCharacteristics::Iec61966 => "11",
            TransferCharacteristics::Bt1361 => "12",
            TransferCharacteristics::Srgb => "13",
            TransferCharacteristics::Bt202010 => "14",
            TransferCharacteristics::Bt202012 => "15",
            TransferCharacteristics::Smpte2084 => "16",
            TransferCharacteristics::Smpte428 => "17",
            TransferCharacteristics::Hlg => "18",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ColorRange {
    Studio, // [0], default
    Full,   // [1] full
}

impl Default for ColorRange {
    fn default() -> Self {
        ColorRange::Studio
    }
}

impl ColorRange {
    pub fn as_str(&self) -> &str {
        match self {
            ColorRange::Studio => "0",
            ColorRange::Full => "1",
        }
    }
}
