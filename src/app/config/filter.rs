use serde::{Deserialize, Serialize};

// Least cursed Rust macro
macro_rules! filter_types {
    // TODO: use $default for something
    ( $( $filter:ident { $( #[prop($name:literal, $default:expr)] $prop:ident : $type:ty , )* } , )* ) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum FilterType {
            $( $filter { $( $prop: $type ),* } ),*
        }

        impl FilterType {
            // TODO: don't allocate
            pub fn properties(&self) -> Vec<FilterProperty> {
                pastey::paste! {
                match self {
                    $(
                    Self::$filter { $( $prop ),* } => {
                        vec![
                            $(
                            FilterProperty {
                                id: _PropertyId::[<$filter $prop:camel>],
                                name: $name,
                                val: PropVal::[<$type:camel>](*$prop),
                            },
                            )*
                        ]
                    }
                    )*
                }
                }
            }
        }

        pastey::paste! {
            #[derive(Debug, Clone, Copy)]
            enum _PropertyId {
                $( $( [<$filter $prop:camel>], )* )*
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct FilterProperty {
            id: _PropertyId,
            name: &'static str,
            pub val: PropVal,
        }

        impl FilterProperty {
            pub fn name(&self) -> &'static str {
                self.name
            }

            pub fn set(&self, filter: &mut FilterType) {
                pastey::paste! {
                match (self.id, self.val, filter) {
                    $( $(
                    (
                        _PropertyId::[<$filter $prop:camel>],
                        PropVal::[<$type:camel>](target),
                        FilterType::$filter { $prop, .. },
                    ) => {
                        *$prop = target;
                    }
                    )* )*

                    (id, val, filter_type) => {
                        tracing::error!(?id, ?val, ?filter_type, "Invalid combination of FilterProperty and FilterType");
                        panic!("Invalid combination of FilterProperty and FilterType. See tracing log for debug info");
                    }
                }
                }
            }
        }
    };
}

filter_types! {
    BassBoost {
        #[prop("Gain", 0.0)]
        gain: f32,
        #[prop("Cutoff", 10000.0)]
        cutoff: f32,
    },
    Shittify {
        #[prop("Strength", 10)]
        strength: i32,
        #[prop("Cutoff", 10000)]
        cutoff: i32,
    },
    Reverb {
        #[prop("Room Size", 0.8)]
        room_size: f32,
        #[prop("Damping", 0.2)]
        damping: f32,
        #[prop("Wet", 0.3)]
        wet: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum PropVal {
    F32(f32),
    I32(i32),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioFilter {
    #[serde(rename = "type")]
    pub filter_type: FilterType,
    #[serde(
        default = "AudioFilter::enabled_default",
        skip_serializing_if = "AudioFilter::is_enabled_default"
    )]
    pub enabled: bool,
    #[serde(skip)]
    pub expanded: bool,
}

impl AudioFilter {
    fn is_enabled_default(v: &bool) -> bool {
        *v
    }

    fn enabled_default() -> bool {
        true
    }

    pub fn name(&self) -> &'static str {
        match self.filter_type {
            FilterType::BassBoost { .. } => "Bass Boost",
            FilterType::Shittify { .. } => "Shittify",
            FilterType::Reverb { .. } => "Reverb",
        }
    }
}
