pub type Gna2Status = i32;

pub const GNA2_STATUS_SUCCESS: Gna2Status = 0;
pub const GNA2_STATUS_WARNING_DEVICE_BUSY: Gna2Status = 1;
pub const GNA2_STATUS_WARNING_ARITHMETIC_SATURATION: Gna2Status = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gna2DeviceVersion(pub u32);

impl Gna2DeviceVersion {
    pub const GMM: Self = Self(0x01);
    pub const V0_9: Self = Self(0x09);
    pub const V1_0: Self = Self(0x10);
    pub const V2_0: Self = Self(0x20);
    pub const V3_0: Self = Self(0x30);
    pub const V3_5: Self = Self(0x35);
    pub const SOFTWARE_EMULATION: Self = Self(u32::MAX);

    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::GMM => "GNA GMM",
            Self::V0_9 => "GNA 0.9",
            Self::V1_0 => "GNA 1.0",
            Self::V2_0 => "GNA 2.0",
            Self::V3_0 => "GNA 3.0",
            Self::V3_5 => "GNA 3.5",
            Self::SOFTWARE_EMULATION => "Software Emulation",
            _ => "Unknown GNA Version",
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2AccelerationMode {
    #[default]
    Auto = 0,
    Software = 1,
    Hardware = 2,
    Avx2 = 3,
    Avx1 = 4,
    Sse4x2 = 5,
    Generic = 6,
    HardwareWithSoftwareFallback = 7,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gna2InstrumentationPoint {
    LibPreprocessing = 0,
    LibSubmission = 1,
    LibProcessing = 2,
    LibExecution = 3,
    LibDeviceRequestReady = 4,
    LibDeviceRequestSent = 5,
    LibDeviceRequestCompleted = 6,
    LibCompletion = 7,
    LibReceived = 8,
    DrvPreprocessing = 9,
    DrvProcessing = 10,
    DrvDeviceRequestCompleted = 11,
    DrvCompletion = 12,
    HwTotalCycles = 13,
    HwStallCycles = 14,
}
