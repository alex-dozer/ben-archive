#[macro_export]
macro_rules! specs {

    ($name:ident, bits = $bits:path, { $($rule:tt)* }) => {{

        #[allow(non_snake_case)]
        mod __ben_specs_helpers {
            pub use $crate::ben_contracts::{Rule, Action};
            pub type Bits = u128;

            #[macro_export]
            macro_rules! __ben_specs_mask {
                ($bits:path; ) => { 0u128 };
                ($bits:path; $head:ident $(, $tail:ident)*) => {
                    ($bits::$head as u128) | $crate::__ben_specs_mask!($bits; $($tail),*)
                };
            }


            #[macro_export]
            macro_rules! __ben_specs_action {
                (PASS)                => { Action::Pass };
                (RouteC)              => { Action::RouteC };
                (RouteD)              => { Action::RouteD };
                (Pause)               => { Action::Pause };
                (Sample($p:literal))  => { Action::Sample($p) };
                (Quarantine($s:literal)) => { Action::Quarantine($s) };
            }


            #[macro_export]
            macro_rules! __ben_specs_rule {

                (ALL($($all:ident),*) & NONE($($none:ident),*) => $act:ident @prio $prio:literal ;) => {
                    Rule { all: $crate::__ben_specs_mask!($crate:: $bits; $($all),*),
                           any: 0,
                           none: $crate::__ben_specs_mask!($crate:: $bits; $($none),*),
                           action: $crate::__ben_specs_action!($act),
                           priority: $prio,
                           id: line!(), }
                };

                (ALL($($all:ident),*) => $act:ident ;) => {
                    Rule { all: $crate::__ben_specs_mask!($crate:: $bits; $($all),*),
                           any: 0, none: 0,
                           action: $crate::__ben_specs_action!($act),
                           priority: 100, id: line!(), }
                };

                (ANY($($any:ident),*) => $act:ident @prio $prio:literal ;) => {
                    Rule { all: 0,
                           any: $crate::__ben_specs_mask!($crate:: $bits; $($any),*),
                           none: 0,
                           action: $crate::__ben_specs_action!($act),
                           priority: $prio, id: line!(), }
                };

                (ANY($($any:ident),*) => $act:ident ;) => {
                    Rule { all: 0,
                           any: $crate::__ben_specs_mask!($crate:: $bits; $($any),*),
                           none: 0,
                           action: $crate::__ben_specs_action!($act),
                           priority: 100, id: line!(), }
                };

                (=> PASS ;) => {
                    Rule { all: 0, any: 0, none: 0, action: Action::Pass, priority: 0, id: line!(), }
                };
            }


            #[macro_export]
            macro_rules! __ben_specs_rules {
                ($bits:path; $($one:tt)*) => {{
                    #[allow(unused_mut)]
                    let mut v: ::std::vec::Vec<Rule> = ::std::vec![
                        $($crate::__ben_specs_rule!($one))*
                    ];
                    v.sort_by(|a,b| b.priority.cmp(&a.priority));
                    v
                }};
            }
        }


        {
            use __ben_specs_helpers::*;
            __ben_specs_rules!($bits; $($rule)*)
        }
    }};
}
