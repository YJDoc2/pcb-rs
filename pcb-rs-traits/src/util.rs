use super::*;

pub fn get_pin_group(
    input: Vec<(ChipPin, &PinMetadata)>,
    output: Vec<(ChipPin, &PinMetadata)>,
) -> Result<ConnectedPins, String> {
    // super basic, there is a single input pin and single output pin
    if input.len() == 1 && output.len() == 1 {
        return Ok(ConnectedPins::Pair {
            source: output[0].0,
            destination: input[0].0,
        });
    }

    let all_tristatable =
        input.iter().all(|(_, md)| md.tristatable) && output.iter().all(|(_, md)| md.tristatable);
    let any_tristatable =
        input.iter().any(|(_, md)| md.tristatable) && output.iter().any(|(_, md)| md.tristatable);

    // TODO validate these in tests later

    // here, there are multiple output pins, and not all are tristated
    if !all_tristatable && output.len() > 1 {
        let temp: Vec<&PinMetadata> = output
            .into_iter()
            .chain(input.into_iter())
            .map(|(_, md)| md)
            .collect();
        return Result::Err(format!(
                "multiple output pins found in a non-tristated pin group : {:#?}\nOnly groups where all pins are tristatable are allowed to have multiple output pins"
                ,temp));
    }

    // I'm not sure if this condition can ever occur, as we check each connected pin pairing,
    // and tristatable mark is like a colour, so that only tristatable pin can be connected to tristatable pins
    // but it is put here as a safety measure
    if !all_tristatable && any_tristatable {
        let temp: Vec<&PinMetadata> = output
            .into_iter()
            .chain(input.into_iter())
            .map(|(_, md)| md)
            .collect();
        return Result::Err(format!(
                "these pins are shorted, but not all are tristatable : {:#?}\nIf any pin a a shourted pin group is tristatable, then all must be tristatable"
                ,temp
            ));
    }

    // single output pin connected to multiple input pins, this is a broadcast
    if output.len() == 1 {
        return Ok(ConnectedPins::Broadcast {
            source: output[0].0,
            destinations: input.into_iter().map(|(p, _)| p).collect(),
        });
    }

    Ok(ConnectedPins::Tristated {
        sources: output.into_iter().map(|(p, _)| p).collect(),
        destinations: input.into_iter().map(|(p, _)| p).collect(),
    })
}
