use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! define_event_type {
    ($( $variant:ident = $val:literal ),* $(,)?) => {
        /// Event type enum matching TBA / the-blue-alliance event_type.py
        /// <https://github.com/the-blue-alliance/the-blue-alliance/blob/master/consts/event_type.py>
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum EventType {
            $( $variant, )*
            Unknown(i64),
        }

        impl From<EventType> for i64 {
            fn from(value: EventType) -> Self {
                match value {
                    $( EventType::$variant => $val, )*
                    EventType::Unknown(v) => v,
                }
            }
        }

        impl From<i64> for EventType {
            fn from(value: i64) -> Self {
                match value {
                    $( $val => EventType::$variant, )*
                    other => EventType::Unknown(other),
                }
            }
        }
    };
}

define_event_type! {
    Regional = 0,
    District = 1,
    DistrictCmp = 2,
    CmpDivision = 3,
    CmpFinals = 4,
    DistrictCmpDivision = 5,
    Foc = 6,
    Remote = 7,
    Offseason = 99,
    Preseason = 100,
}

impl EventType {
    pub const CMP_EVENT_TYPES: &[EventType] = &[EventType::CmpDivision, EventType::CmpFinals];

    pub const SEASON_EVENT_TYPES: &[EventType] = &[
        EventType::Regional,
        EventType::District,
        EventType::DistrictCmpDivision,
        EventType::DistrictCmp,
        EventType::CmpDivision,
        EventType::CmpFinals,
        EventType::Foc,
        EventType::Remote,
    ];

    pub fn is_championship(self) -> bool {
        Self::CMP_EVENT_TYPES.contains(&self)
    }

    pub fn is_official(self) -> bool {
        Self::SEASON_EVENT_TYPES.contains(&self)
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(EventType::from(i64::deserialize(deserializer)?))
    }
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        i64::from(*self).serialize(serializer)
    }
}
