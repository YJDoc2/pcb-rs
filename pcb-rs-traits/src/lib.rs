use std::any::Any;
use std::collections::HashMap;

mod util;
pub use util::get_pin_group;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinType {
    Input,
    Output,
    IO,
}
#[derive(Debug, Clone, Copy)]
/// This will store the metadata of the pin, for the encompassing
/// module (usually generated using pcb!) to use. Reason that the data_type is
/// &'static str is that when deriving the Chip using Chip derive macro,
/// or even when hand-implementing ChipInterface, the data type of the
/// pin will be known in advance. Name is not stored here as it will be the key of hashmap
pub struct PinMetadata {
    pub pin_type: PinType,
    pub data_type: &'static str,
    pub tristatable: bool,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
/// Used to represent a specific pin of a specific chip
/// in pcb! generated struct. Both fields are 'static , because
/// when generating we know the names, and thus can be stored as static strings
pub struct ChipPin {
    pub chip: &'static str,
    pub pin: &'static str,
}

#[derive(Debug)]
// NOTE: if you don't understand rest of this comment it is fine, even I'm a bit confused after writing it!
// Note that in terms of the chip, a pin will be `output`, i.e. it will give out some data after tick()
// ant the pins which are connected to such pins will be `input` pins, as they take the data before tick()
// for the encompassing hardware module, this definition is kind of reversed in a sense, the pin which give out output are
// are the input pins (termed source here) as we take their value after tick() and give it to some other pins, i.e.
// they provide some input to other chips, and the pins which take it are output pins (termed destination),
// i.e. they are supposed to receive the output. It is confusing, so here I just called it source and destination
/// This enum represents three types of pin connections possible
pub enum ConnectedPins {
    /// indicates a one-to-one connections of a pin from a chip to another pin of a chip
    Pair {
        source: ChipPin,
        destination: ChipPin,
    },
    /// indicates a group of shorted pins ,where a single pin is output type,
    /// and its value is broadcasted to rest of the pins, which are either input or io types
    Broadcast {
        source: ChipPin,
        destinations: Vec<ChipPin>,
    },
    /// indicates the group of connected tristated pins, with set of output/io types pins
    /// connected to set of input/io pins
    Tristated {
        sources: Vec<ChipPin>,
        destinations: Vec<ChipPin>,
    },
}

impl PinMetadata {
    pub fn is_connectable(&self, other: &PinMetadata) -> bool {
        let both_input =
            matches!(self.pin_type, PinType::Input) && matches!(other.pin_type, PinType::Input);
        let both_output =
            matches!(self.pin_type, PinType::Output) && matches!(other.pin_type, PinType::Output);
        let both_tristatable = self.tristatable ^ other.tristatable;
        let both_same_type = self.data_type == other.data_type;

        // for pins to be connectable, both should NOT be input, both should NOT be output
        // and both either should or should not be tristatable, the xor gives true if one is and one isn't
        // so we check for it being false as well, and both should be of same type

        return !both_input && !both_output && !both_tristatable && both_same_type;
    }
}

impl std::fmt::Display for PinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PinType::Input => "Input",
                PinType::Output => "Output",
                PinType::IO => "IO",
            }
        )
    }
}

// Ideally we should enforce that the value type should be Any+Clone,
// as we will call clone in the set_pin_value,
// but adding clone to is not allowed due to object safety
// so instead we only use Any, and depend on the fact that
// before using clone, we will be converting the trait object back to concrete
// so clone can be called, and if the type itself does not implement clone,
// that call will fail at compile time

/// This is the interface which should be exposed by the chip struct,
/// and will be used by the pcb module. This is meant to be implemented
/// by the #[Derive(Chip)] macro, but can also be manually implemented if needed
pub trait ChipInterface {
    /// gives a mapping from pin name to pin metadata
    fn get_pin_list(&self) -> HashMap<&'static str, PinMetadata>;

    /// returns value of a specific pin, typecasted to Any
    fn get_pin_value(&self, name: &str) -> Option<Box<dyn Any>>;

    /// sets value of a specific pin, from the given reference
    fn set_pin_value(&mut self, name: &str, val: &dyn Any);

    // The reason to include it in Chip interface, rather than anywhere else,
    // is that I couldn't find a more elegant solution that can either directly
    // implement on pin values which are typecasted to dyn Any. Thus the only way
    // that we can absolutely make sure if a pin is tristated or not is in the
    // Chip-level rather than the pin level. One major issue is that the data of
    // which type the pin is is only available in the Chip derive macro, and cannot be
    // used by the encompassing module in a way that will allow its usage in user programs
    // which does not depend on syn/quote libs.
    /// This is used to check if a tristatable pin is tristated or not
    fn is_pin_tristated(&self, name: &str) -> bool;

    /// This returns if the io pin is in input mode or not, and false for other pins
    fn in_input_mode(&self, name: &str) -> bool;
}

/// This is intended to be implemented manually by user
/// of the library. This provides the functionality of
/// actually "running" the logic of the chip
pub trait Chip {
    /// this will be called on each clock tick by encompassing module (usually derived by pcb! macro)
    /// and should contain the logic which is to be "implemented" by the chip.
    ///
    /// Before calling this function the values of input pins wil be updated according to
    /// which other pins are connected to those, but does not guarantee
    /// what value will be set in case if multiple output pins are connected to a single input pin.
    ///
    /// After calling this function, and before the next call of this function, the values of
    /// output pins will be gathered by the encompassing module, to be given to the input pins before
    /// next call of this.
    ///
    /// Thus ideally this function should check values of its input pins, take according actions and
    /// set values of output pins. Although in case the chip itself needs to do something else, (eg logging etc)
    /// it can simply do that and not set any pin to output in its struct declaration.
    fn tick(&mut self) -> ();
}

/// This trait is used to create trait objects to store in the pcb created by the pbc! macro
pub trait HardwareModule: ChipInterface + Chip {}

impl<T> HardwareModule for T where T: ChipInterface + Chip {}
