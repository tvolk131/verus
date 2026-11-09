use super::*;

fn record(typ: &Typ, datatypes: &[Datatype], depth: usize, leaves: &mut usize) -> bool {
    if depth == 0 {
        return false;
    }
    let Some(fields) = records::shape::fields(typ, datatypes) else { return false };
    fields.iter().all(|(_, field)| {
        if scalar(field) {
            if *leaves == 0 {
                return false;
            }
            *leaves -= 1;
            true
        } else {
            record(field, datatypes, depth - 1, leaves)
        }
    })
}

pub(super) fn option_record(typ: &Typ, datatypes: &[Datatype]) -> bool {
    option(typ).is_some_and(|payload| record(payload, datatypes, 4, &mut 16))
}

pub(super) fn shared_roots(function: &Function, datatypes: &[Datatype]) -> bool {
    let parameters = &function.x.params;
    let mut roots = 0;
    let mut leaves = 16;
    parameters.len() <= 8
        && scalar(&function.x.inner_ret.x.typ)
        && parameters.iter().all(|parameter| {
            if let Some(target) = trait_getters::shared(&parameter.x.typ) {
                roots += 1;
                record(target, datatypes, 4, &mut leaves)
            } else {
                scalar(&parameter.x.typ)
            }
        })
        && roots >= 2
}
