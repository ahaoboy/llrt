// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use std::sync::atomic::Ordering;

use rquickjs::{atom::PredefinedAtom, prelude::Func, Ctx, Object, Result};

use chrono::Utc;

use llrt_modules::perf_hooks::new_performance;

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    let performance = new_performance(ctx)?;
    globals.set("performance", performance)?;

    Ok(())
}
