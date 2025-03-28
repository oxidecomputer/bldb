// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(crate) fn cpuid(leaf: u32, subleaf: u32) -> x86::cpuid::CpuIdResult {
    x86::cpuid::native_cpuid::cpuid_count(leaf, subleaf)
}

/// Returns information about the current processor and its
/// package.
pub(crate) fn cpuinfo() -> Option<(u8, u8, u8, Option<u32>)> {
    let cpuid = x86::cpuid::CpuId::new();
    let features = cpuid.get_feature_info()?;
    let family = features.family_id();
    let ext = cpuid.get_extended_processor_and_feature_identifiers()?;
    let pkg_type = (family > 0x10).then_some(ext.pkg_type());
    Some((family, features.model_id(), features.stepping_id(), pkg_type))
}
