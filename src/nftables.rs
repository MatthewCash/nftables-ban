use anyhow::Result;
use nftables::{
    expr::Expression,
    schema::{
        Element, FlushObject, NfCmd, NfListObject, NfObject, Nftables, Set, SetType, SetTypeValue,
    },
    types::NfFamily,
};
use serde::Deserialize;
use std::{borrow::Cow, net::Ipv4Addr};

#[derive(Clone, Debug, Deserialize)]
pub struct NftablesConfig {
    set_table: String,
    set_name: String,
}

pub fn flush_set(config: &NftablesConfig) -> Result<()> {
    let rules = [NfObject::CmdObject(NfCmd::Flush(FlushObject::Set(
        Box::new(Set {
            family: NfFamily::INet,
            table: config.set_table.clone().into(),
            name: config.set_name.clone().into(),
            handle: None,
            set_type: SetTypeValue::Single(SetType::Ipv4Addr),
            policy: None,
            flags: None,
            elem: None,
            timeout: None,
            gc_interval: None,
            size: None,
            comment: None,
        }),
    )))];

    Ok(nftables::helper::apply_ruleset(&Nftables {
        objects: Cow::Borrowed(&rules),
    })?)
}

pub fn add_addrs_to_set(config: &NftablesConfig, addrs: &[Ipv4Addr]) -> Result<()> {
    if addrs.is_empty() {
        return Ok(());
    }

    let addr_elements = addrs
        .iter()
        .map(|a| Expression::String(a.to_string().into()))
        .collect::<Vec<_>>();

    let rules = [NfObject::CmdObject(NfCmd::Add(NfListObject::Element(
        Element {
            family: NfFamily::INet,
            table: config.set_table.clone().into(),
            name: config.set_name.clone().into(),
            elem: Cow::Borrowed(&addr_elements),
        },
    )))];

    Ok(nftables::helper::apply_ruleset(&Nftables {
        objects: Cow::Borrowed(&rules),
    })?)
}

pub fn remove_addrs_from_set(config: &NftablesConfig, addrs: &[Ipv4Addr]) -> Result<()> {
    if addrs.is_empty() {
        return Ok(());
    }

    let addr_elements = addrs
        .iter()
        .map(|a| Expression::String(a.to_string().into()))
        .collect::<Vec<_>>();

    let rules = [NfObject::CmdObject(NfCmd::Delete(NfListObject::Element(
        Element {
            family: NfFamily::INet,
            table: config.set_table.clone().into(),
            name: config.set_name.clone().into(),
            elem: Cow::Borrowed(&addr_elements),
        },
    )))];

    Ok(nftables::helper::apply_ruleset(&Nftables {
        objects: Cow::Borrowed(&rules),
    })?)
}
