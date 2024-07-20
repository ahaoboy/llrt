// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use crate::ModuleInfo;
use llrt_utils::module::export_default;
use once_cell::sync::Lazy;
use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    prelude::Func,
    Ctx, Object, Result,
};
use std::time::SystemTime;

static START_TIME: Lazy<SystemTime> = Lazy::new(SystemTime::now);

fn now() -> usize {
    SystemTime::now()
        .duration_since(*START_TIME)
        .expect("Time went backwards")
        .as_millis() as usize
}

pub struct PerfHooksModule;

impl ModuleDef for PerfHooksModule {
    fn declare(declare: &Declarations<'_>) -> Result<()> {
        declare.declare("performance")?;
        declare.declare("default")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        export_default(ctx, exports, |default| {
            let now_fn = Func::from(now);
            let performance = Object::new(ctx.clone())?;
            performance.set("now", now_fn)?;
            default.set("performance", performance)?;
            Ok(())
        })
    }
}

impl From<PerfHooksModule> for ModuleInfo<PerfHooksModule> {
    fn from(val: PerfHooksModule) -> Self {
        ModuleInfo {
            name: "perf_hooks",
            module: val,
        }
    }
}
