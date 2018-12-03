use wasmi::{Error as WasmiError, Trap, TrapKind};

#[derive(Debug, PartialEq, Eq)]
/// Error that can occur while preparing or executing wasm smart-contract.
pub enum PrepareError {
    /// Error happened while serializing the module.
    Serialization,

    /// Error happened while deserializing the module.
    Deserialization,

    /// Internal memory declaration has been found in the module.
    InternalMemoryDeclared,

    /// Gas instrumentation failed.
    ///
    /// This most likely indicates the module isn't valid.
    GasInstrumentation,

    /// Stack instrumentation failed.
    ///
    /// This  most likely indicates the module isn't valid.
    StackHeightInstrumentation,

    /// Error happened during invocation of the contract's entrypoint.
    ///
    /// Most likely because of trap.
    Invoke,

    /// Error happened during instantiation.
    ///
    /// This might indicate that `start` function trapped, or module isn't
    /// instantiable and/or unlinkable.
    Instantiate,

    /// Memory creation error.
    ///
    /// This might happen when the memory import has invalid descriptor or
    /// requested too much resources.
    Memory,
}

/// User trap in native code
#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// Storage read error
    StorageReadError,
    /// Storage update error
    StorageUpdateError,
    /// Memory access violation
    MemoryAccessViolation,
    /// Native code returned incorrect value
    InvalidReturn,
    /// Invalid gas state inside interpreter
    InvalidGasState,
    /// Query of the balance resulted in an error
    BalanceQueryError,
    /// Failed allocation
    AllocationFailed,
    /// Gas limit reached
    GasLimit,
    /// Unknown runtime function
    Unknown,
    /// Passed string had invalid utf-8 encoding
    BadUtf8,
    /// Log event error
    Log,
    /// Other error in native code
    Other,
    /// Syscall signature mismatch
    InvalidSyscall,
    /// Unreachable instruction encountered
    Unreachable,
    /// Invalid virtual call
    InvalidVirtualCall,
    /// Division by zero
    DivisionByZero,
    /// Invalid conversion to integer
    InvalidConversionToInt,
    /// Stack overflow
    StackOverflow,
    /// Panic with message
    Panic(String),
}

impl wasmi::HostError for RuntimeError {}

impl From<Trap> for RuntimeError {
    fn from(trap: Trap) -> Self {
        match *trap.kind() {
            TrapKind::Unreachable => RuntimeError::Unreachable,
            TrapKind::MemoryAccessOutOfBounds => RuntimeError::MemoryAccessViolation,
            TrapKind::TableAccessOutOfBounds | TrapKind::ElemUninitialized => {
                RuntimeError::InvalidVirtualCall
            }
            TrapKind::DivisionByZero => RuntimeError::DivisionByZero,
            TrapKind::InvalidConversionToInt => RuntimeError::InvalidConversionToInt,
            TrapKind::UnexpectedSignature => RuntimeError::InvalidVirtualCall,
            TrapKind::StackOverflow => RuntimeError::StackOverflow,
            TrapKind::Host(_) => RuntimeError::Other,
        }
    }
}

impl From<WasmiError> for RuntimeError {
    fn from(err: WasmiError) -> Self {
        match err {
            WasmiError::Value(_) => RuntimeError::InvalidSyscall,
            WasmiError::Memory(_) => RuntimeError::MemoryAccessViolation,
            _ => RuntimeError::Other,
        }
    }
}

impl ::std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            RuntimeError::StorageReadError => write!(f, "Storage read error"),
            RuntimeError::StorageUpdateError => write!(f, "Storage update error"),
            RuntimeError::MemoryAccessViolation => write!(f, "Memory access violation"),
            RuntimeError::InvalidGasState => write!(f, "Invalid gas state"),
            RuntimeError::BalanceQueryError => write!(f, "Balance query resulted in an error"),
            RuntimeError::InvalidReturn => write!(f, "Invalid return value"),
            RuntimeError::Unknown => write!(f, "Unknown runtime function invoked"),
            RuntimeError::AllocationFailed => write!(f, "Memory allocation failed (OOM)"),
            RuntimeError::BadUtf8 => write!(f, "String encoding is bad utf-8 sequence"),
            RuntimeError::GasLimit => write!(f, "Invocation resulted in gas limit violated"),
            RuntimeError::Log => write!(f, "Error occured while logging an event"),
            RuntimeError::InvalidSyscall => write!(f, "Invalid syscall signature encountered at runtime"),
            RuntimeError::Other => write!(f, "Other unspecified error"),
            RuntimeError::Unreachable => write!(f, "Unreachable instruction encountered"),
            RuntimeError::InvalidVirtualCall => write!(f, "Invalid virtual call"),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::StackOverflow => write!(f, "Stack overflow"),
            RuntimeError::InvalidConversionToInt => write!(f, "Invalid conversion to integer"),
            RuntimeError::Panic(ref msg) => write!(f, "Panic: {}", msg),
        }
    }
}

/// Wrapped error
#[derive(Debug)]
pub enum Error {
    /// Method name can't be decoded to UTF8.
    BadUtf8,

    /// Method name is empty.
    EmptyMethodName,

    /// Method is private, because it starts with '_'.
    PrivateMethod,

    Runtime(RuntimeError),

    Prepare(PrepareError),

    Interpreter(WasmiError),

    Trap(Trap),
}

impl From<WasmiError> for Error {
    fn from(e: WasmiError) -> Self {
        Error::Interpreter(e)
    }
}

impl From<Trap> for Error {
    fn from(e: Trap) -> Self {
        Error::Trap(e)
    }
}

impl From<RuntimeError> for Error {
    fn from(e: RuntimeError) -> Self {
        Error::Runtime(e)
    }
}

#[derive(Clone, Debug)]
pub struct ExecutionParams {
    pub config: Config,
}

// TODO: Extract it to the root of the crate
#[derive(Clone, Debug)]
pub struct Config {
    /// Gas cost of a growing memory by single page.
    pub grow_mem_cost: u32,

    /// Gas cost of a regular operation.
    pub regular_op_cost: u32,

    /// Gas cost per one byte returned.
    pub return_data_per_byte_cost: u32,

    /// How tall the stack is allowed to grow?
    ///
    /// See https://wiki.parity.io/WebAssembly-StackHeight to find out
    /// how the stack frame cost is calculated.
    pub max_stack_height: u32,

    /// What is the maximal memory pages amount is allowed to have for
    /// a contract.
    pub max_memory_pages: u32,

    /// Gas limit of the one contract call
    pub gas_limit: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            grow_mem_cost: 1,
            regular_op_cost: 1,
            return_data_per_byte_cost: 1,
            max_stack_height: 64 * 1024,
            max_memory_pages: 16,
            gas_limit: 128 * 1024,
        }
    }
}