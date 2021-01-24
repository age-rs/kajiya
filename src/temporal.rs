use std::sync::Arc;

use slingshot::{
    rg::{self, ImportExportToRenderGraph, Resource},
    vk_sync,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TemporalResourceState {
    Default,
    Imported,
    Exported,
}

pub struct Temporal<Res: Resource> {
    resource: Arc<Res>,
    access_type: vk_sync::AccessType,
    last_rg_handle: Option<rg::ExportedHandle<Res>>,
    state: TemporalResourceState,
}

#[allow(dead_code)]
impl<Res: Resource> Temporal<Res> {
    pub fn new(resource: Arc<Res>) -> Self {
        Self {
            resource,
            access_type: vk_sync::AccessType::Nothing,
            last_rg_handle: None,
            state: TemporalResourceState::Default,
        }
    }

    pub fn last_rg_handle(&self) -> Option<rg::ExportedHandle<Res>> {
        self.last_rg_handle
    }
}

pub trait RgTemporalExt {
    fn import_temporal<Res: Resource + ImportExportToRenderGraph>(
        &mut self,
        temporal: &mut Temporal<Res>,
    ) -> rg::Handle<Res>;

    fn export_temporal<Res: Resource + ImportExportToRenderGraph>(
        &mut self,
        handle: rg::Handle<Res>,
        temporal: &mut Temporal<Res>,
        access_type: vk_sync::AccessType,
    ) -> rg::ExportedHandle<Res>;
}

pub trait RetiredRgTemporalExt {
    fn retire_temporal<Res: Resource + ImportExportToRenderGraph>(
        &self,
        temporal: &mut Temporal<Res>,
    );
}

impl RgTemporalExt for rg::RenderGraph {
    fn import_temporal<Res: Resource + ImportExportToRenderGraph>(
        &mut self,
        temporal: &mut Temporal<Res>,
    ) -> rg::Handle<Res> {
        temporal.state = TemporalResourceState::Imported;
        self.import(temporal.resource.clone(), temporal.access_type)
    }

    fn export_temporal<Res: Resource + ImportExportToRenderGraph>(
        &mut self,
        handle: rg::Handle<Res>,
        temporal: &mut Temporal<Res>,
        access_type: vk_sync::AccessType,
    ) -> rg::ExportedHandle<Res> {
        assert_eq!(temporal.state, TemporalResourceState::Imported);
        let exported_handle = self.export(handle, access_type);
        temporal.last_rg_handle = Some(exported_handle);
        temporal.state = TemporalResourceState::Exported;
        exported_handle
    }
}

impl RetiredRgTemporalExt for rg::RetiredRenderGraph {
    fn retire_temporal<Res: Resource + ImportExportToRenderGraph>(
        &self,
        temporal: &mut Temporal<Res>,
    ) {
        if let Some(handle) = temporal.last_rg_handle.take() {
            assert_eq!(temporal.state, TemporalResourceState::Exported);
            temporal.access_type = self.exported_resource(handle).1;
            temporal.state = TemporalResourceState::Default;
        }
    }
}
