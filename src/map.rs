//! Some of this module (meshing and shader code, etc) is based on code by Morgan McGuire, which
//! appeared in his blog Casual Effects
//! [here](http://casual-effects.blogspot.com/2014/04/fast-terrain-rendering-with-continuous.html).
//! The original source is available here: https://github.com/morgan3d/misc/tree/master/terrain
//!
//! The license of the original source follows:
//!
//! ```
//! MIT License
//!
//! Copyright (c) 2016 morgan3d
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```
//!
use crate::{AppState, STATE_STAGE};
use bevy::app::{AppBuilder, Plugin};
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::tasks::AsyncComputeTaskPool;
use std::future::Future;
use std::io::Read;
use std::pin::Pin;

pub mod mesh;
pub mod shader;

pub struct RomeMapPlugin;

impl Plugin for RomeMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        dbg!("added shader map material");
        app.add_asset::<HeightMap>()
            .add_asset::<shader::MapMaterial>()
            .add_startup_system(shader::setup.system())
            .on_state_update(
                STATE_STAGE,
                AppState::InGame,
                shader::update_time.system(),
            );
    }
}

#[derive(TypeUuid)]
#[uuid = "7b7c08b3-986e-49d8-85da-107024f177f1"]
pub struct HeightMap(pub rome_map::Map);

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct HeightMapLoader {
    pub task_pool: AsyncComputeTaskPool,
}

impl AssetLoader for HeightMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        ctx: &'a mut LoadContext,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            // TODO no block
            let asset = self.task_pool.scope(|scope| {
                scope.spawn(async {
                    println!("Unzipping");

                    let mut decoder = zstd::Decoder::new(&*bytes).unwrap();
                    let mut unzipped = Vec::new();
                    decoder.read_to_end(&mut unzipped).unwrap();

                    println!("Deserializing");
                    let map: rome_map::Map = bincode::deserialize(&unzipped).unwrap();

                    println!("Done loading heightmap");

                    HeightMap(map)
                })
            });

            ctx.set_default_asset(LoadedAsset::new(asset.into_iter().next().unwrap()));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mapdat"]
    }
}
