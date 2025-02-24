pub mod box_shadow;
mod pipeline;
mod render_pass;
mod ui_material_pipeline;
pub mod ui_texture_slice_pipeline;

#[cfg(feature = "bevy_ui_debug")]
mod debug_overlay;

use crate::widget::ImageNode;
use crate::{
    BackgroundColor, BorderColor, BoxShadowSamples, CalculatedClip, ComputedNode,
    ComputedNodeTarget, DefaultUiCamera, Outline, ResolvedBorderRadius, TextShadow, UiAntiAlias,
    UiTargetCamera,
};
use bevy_app::prelude::*;
use bevy_asset::{load_internal_asset, weak_handle, AssetEvent, AssetId, Assets, Handle};
use bevy_color::{Alpha, ColorToComponents, LinearRgba};
use bevy_core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy_core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy_core_pipeline::{core_2d::Camera2d, core_3d::Camera3d};
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_image::prelude::*;
use bevy_math::{FloatOrd, Mat4, Rect, UVec4, Vec2, Vec3, Vec3Swizzles, Vec4Swizzles};
use bevy_render::render_graph::{NodeRunError, RenderGraphContext};
use bevy_render::render_phase::ViewSortedRenderPhases;
use bevy_render::renderer::RenderContext;
use bevy_render::sync_world::MainEntity;
use bevy_render::texture::TRANSPARENT_IMAGE_HANDLE;
use bevy_render::view::RetainedViewEntity;
use bevy_render::{
    camera::Camera,
    render_asset::RenderAssets,
    render_graph::{Node as RenderGraphNode, RenderGraph},
    render_phase::{sort_phase_system, AddRenderCommand, DrawFunctions},
    render_resource::*,
    renderer::{RenderDevice, RenderQueue},
    view::{ExtractedView, ViewUniforms},
    Extract, RenderApp, RenderSet,
};
use bevy_render::{
    render_phase::{PhaseItem, PhaseItemExtraIndex},
    sync_world::{RenderEntity, TemporaryRenderEntity},
    texture::GpuImage,
    view::InheritedVisibility,
    ExtractSchedule, Render,
};
use bevy_sprite::{BorderRect, SpriteAssetEvents};
#[cfg(feature = "bevy_ui_debug")]
pub use debug_overlay::UiDebugOptions;

use crate::{Display, Node};
use bevy_platform_support::collections::{HashMap, HashSet};
use bevy_text::{ComputedTextBlock, PositionedGlyph, TextColor, TextLayoutInfo};
use bevy_transform::components::GlobalTransform;
use box_shadow::BoxShadowPlugin;
use bytemuck::{Pod, Zeroable};
use core::ops::Range;
use graph::{NodeUi, SubGraphUi};
pub use pipeline::*;
pub use render_pass::*;
pub use ui_material_pipeline::*;
use ui_texture_slice_pipeline::UiTextureSlicerPlugin;

pub mod graph {
    use bevy_render::render_graph::{RenderLabel, RenderSubGraph};

    #[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
    pub struct SubGraphUi;

    #[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
    pub enum NodeUi {
        UiPass,
    }
}

/// Z offsets of "extracted nodes" for a given entity. These exist to allow rendering multiple "extracted nodes"
/// for a given source entity (ex: render both a background color _and_ a custom material for a given node).
///
/// When possible these offsets should be defined in _this_ module to ensure z-index coordination across contexts.
/// When this is _not_ possible, pick a suitably unique index unlikely to clash with other things (ex: `0.1826823` not `0.1`).
///
/// Offsets should be unique for a given node entity to avoid z fighting.
/// These should pretty much _always_ be larger than -0.5 and smaller than 0.5 to avoid clipping into nodes
/// above / below the current node in the stack.
///
/// A z-index of 0.0 is the baseline, which is used as the primary "background color" of the node.
///
/// Note that nodes "stack" on each other, so a negative offset on the node above could clip _into_
/// a positive offset on a node below.
pub mod stack_z_offsets {
    pub const BOX_SHADOW: f32 = -0.1;
    pub const TEXTURE_SLICE: f32 = 0.0;
    pub const NODE: f32 = 0.0;
    pub const MATERIAL: f32 = 0.18267;
}

pub const UI_SHADER_HANDLE: Handle<Shader> = weak_handle!("7d190d05-545b-42f5-bd85-22a0da85b0f6");

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderUiSystem {
    ExtractCameraViews,
    ExtractBoxShadows,
    ExtractBackgrounds,
    ExtractImages,
    ExtractTextureSlice,
    ExtractBorders,
    ExtractTextShadows,
    ExtractText,
    ExtractDebug,
}

pub fn build_ui_render(app: &mut App) {
    load_internal_asset!(app, UI_SHADER_HANDLE, "ui.wgsl", Shader::from_wgsl);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<SpecializedRenderPipelines<UiPipeline>>()
        .init_resource::<ImageNodeBindGroups>()
        .init_resource::<UiMeta>()
        .init_resource::<ExtractedUiNodes>()
        .allow_ambiguous_resource::<ExtractedUiNodes>()
        .init_resource::<DrawFunctions<TransparentUi>>()
        .init_resource::<ViewSortedRenderPhases<TransparentUi>>()
        .add_render_command::<TransparentUi, DrawUi>()
        .configure_sets(
            ExtractSchedule,
            (
                RenderUiSystem::ExtractCameraViews,
                RenderUiSystem::ExtractBoxShadows,
                RenderUiSystem::ExtractBackgrounds,
                RenderUiSystem::ExtractImages,
                RenderUiSystem::ExtractTextureSlice,
                RenderUiSystem::ExtractBorders,
                RenderUiSystem::ExtractTextShadows,
                RenderUiSystem::ExtractText,
                RenderUiSystem::ExtractDebug,
            )
                .chain(),
        )
        .add_systems(
            ExtractSchedule,
            (
                extract_ui_camera_view.in_set(RenderUiSystem::ExtractCameraViews),
                extract_uinode_background_colors.in_set(RenderUiSystem::ExtractBackgrounds),
                extract_uinode_images.in_set(RenderUiSystem::ExtractImages),
                extract_uinode_borders.in_set(RenderUiSystem::ExtractBorders),
                extract_text_shadows.in_set(RenderUiSystem::ExtractTextShadows),
                extract_text_sections.in_set(RenderUiSystem::ExtractText),
                #[cfg(feature = "bevy_ui_debug")]
                debug_overlay::extract_debug_overlay.in_set(RenderUiSystem::ExtractDebug),
            ),
        )
        .add_systems(
            Render,
            (
                queue_uinodes.in_set(RenderSet::Queue),
                sort_phase_system::<TransparentUi>.in_set(RenderSet::PhaseSort),
                prepare_uinodes.in_set(RenderSet::PrepareBindGroups),
            ),
        );

    // Render graph
    let ui_graph_2d = get_ui_graph(render_app);
    let ui_graph_3d = get_ui_graph(render_app);
    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();

    if let Some(graph_2d) = graph.get_sub_graph_mut(Core2d) {
        graph_2d.add_sub_graph(SubGraphUi, ui_graph_2d);
        graph_2d.add_node(NodeUi::UiPass, RunUiSubgraphOnUiViewNode);
        graph_2d.add_node_edge(Node2d::EndMainPass, NodeUi::UiPass);
        graph_2d.add_node_edge(Node2d::EndMainPassPostProcessing, NodeUi::UiPass);
        graph_2d.add_node_edge(NodeUi::UiPass, Node2d::Upscaling);
    }

    if let Some(graph_3d) = graph.get_sub_graph_mut(Core3d) {
        graph_3d.add_sub_graph(SubGraphUi, ui_graph_3d);
        graph_3d.add_node(NodeUi::UiPass, RunUiSubgraphOnUiViewNode);
        graph_3d.add_node_edge(Node3d::EndMainPass, NodeUi::UiPass);
        graph_3d.add_node_edge(Node3d::EndMainPassPostProcessing, NodeUi::UiPass);
        graph_3d.add_node_edge(NodeUi::UiPass, Node3d::Upscaling);
    }

    app.add_plugins(UiTextureSlicerPlugin);
    app.add_plugins(BoxShadowPlugin);
}

fn get_ui_graph(render_app: &mut SubApp) -> RenderGraph {
    let ui_pass_node = UiPassNode::new(render_app.world_mut());
    let mut ui_graph = RenderGraph::default();
    ui_graph.add_node(NodeUi::UiPass, ui_pass_node);
    ui_graph
}

pub struct ExtractedUiNode {
    pub stack_index: u32,
    pub color: LinearRgba,
    pub rect: Rect,
    pub image: AssetId<Image>,
    pub clip: Option<Rect>,
    /// Render world entity of the extracted camera corresponding to this node's target camera.
    pub extracted_camera_entity: Entity,
    pub item: ExtractedUiItem,
    pub main_entity: MainEntity,
    pub render_entity: Entity,
}

/// The type of UI node.
/// This is used to determine how to render the UI node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeType {
    Rect,
    Border,
}

pub enum ExtractedUiItem {
    Node {
        atlas_scaling: Option<Vec2>,
        flip_x: bool,
        flip_y: bool,
        /// Border radius of the UI node.
        /// Ordering: top left, top right, bottom right, bottom left.
        border_radius: ResolvedBorderRadius,
        /// Border thickness of the UI node.
        /// Ordering: left, top, right, bottom.
        border: BorderRect,
        node_type: NodeType,
        transform: Mat4,
    },
    /// A contiguous sequence of text glyphs from the same section
    Glyphs {
        /// Indices into [`ExtractedUiNodes::glyphs`]
        range: Range<usize>,
    },
}

pub struct ExtractedGlyph {
    pub transform: Mat4,
    pub rect: Rect,
}

#[derive(Resource, Default)]
pub struct ExtractedUiNodes {
    pub uinodes: Vec<ExtractedUiNode>,
    pub glyphs: Vec<ExtractedGlyph>,
}

impl ExtractedUiNodes {
    pub fn clear(&mut self) {
        self.uinodes.clear();
        self.glyphs.clear();
    }
}

#[derive(SystemParam)]
pub struct UiCameraMap<'w, 's> {
    mapping: Query<'w, 's, RenderEntity>,
}

impl<'w, 's> UiCameraMap<'w, 's> {
    /// Get the default camera and create the mapper
    pub fn get_mapper(&'w self) -> UiCameraMapper<'w, 's> {
        UiCameraMapper {
            mapping: &self.mapping,
            camera_entity: Entity::PLACEHOLDER,
            render_entity: Entity::PLACEHOLDER,
        }
    }
}

pub struct UiCameraMapper<'w, 's> {
    mapping: &'w Query<'w, 's, RenderEntity>,
    camera_entity: Entity,
    render_entity: Entity,
}

impl<'w, 's> UiCameraMapper<'w, 's> {
    /// Returns the render entity corresponding to the given `UiTargetCamera` or the default camera if `None`.
    pub fn map(&mut self, computed_target: &ComputedNodeTarget) -> Option<Entity> {
        let camera_entity = computed_target.camera;
        if self.camera_entity != camera_entity {
            let Ok(new_render_camera_entity) = self.mapping.get(camera_entity) else {
                return None;
            };
            self.render_entity = new_render_camera_entity;
            self.camera_entity = camera_entity;
        }

        Some(self.render_entity)
    }

    pub fn current_camera(&self) -> Entity {
        self.camera_entity
    }
}

/// A [`RenderGraphNode`] that executes the UI rendering subgraph on the UI
/// view.
struct RunUiSubgraphOnUiViewNode;

impl RenderGraphNode for RunUiSubgraphOnUiViewNode {
    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        _: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        // Fetch the UI view.
        let Some(mut render_views) = world.try_query::<&UiCameraView>() else {
            return Ok(());
        };
        let Ok(default_camera_view) = render_views.get(world, graph.view_entity()) else {
            return Ok(());
        };

        // Run the subgraph on the UI view.
        graph.run_sub_graph(SubGraphUi, vec![], Some(default_camera_view.0))?;
        Ok(())
    }
}

pub fn extract_uinode_background_colors(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            &ComputedNodeTarget,
            &BackgroundColor,
        )>,
    >,
    camera_map: Extract<UiCameraMap>,
) {
    let mut camera_mapper = camera_map.get_mapper();

    for (entity, uinode, transform, inherited_visibility, clip, camera, background_color) in
        &uinode_query
    {
        // Skip invisible backgrounds
        if !inherited_visibility.get()
            || background_color.0.is_fully_transparent()
            || uinode.is_empty()
        {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(camera) else {
            continue;
        };

        extracted_uinodes.uinodes.push(ExtractedUiNode {
            render_entity: commands.spawn(TemporaryRenderEntity).id(),
            stack_index: uinode.stack_index,
            color: background_color.0.into(),
            rect: Rect {
                min: Vec2::ZERO,
                max: uinode.size,
            },
            clip: clip.map(|clip| clip.clip),
            image: AssetId::default(),
            extracted_camera_entity,
            item: ExtractedUiItem::Node {
                atlas_scaling: None,
                transform: transform.compute_matrix(),
                flip_x: false,
                flip_y: false,
                border: uinode.border(),
                border_radius: uinode.border_radius(),
                node_type: NodeType::Rect,
            },
            main_entity: entity.into(),
        });
    }
}

pub fn extract_uinode_images(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            &ComputedNodeTarget,
            &ImageNode,
        )>,
    >,
    camera_map: Extract<UiCameraMap>,
) {
    let mut camera_mapper = camera_map.get_mapper();
    for (entity, uinode, transform, inherited_visibility, clip, camera, image) in &uinode_query {
        // Skip invisible images
        if !inherited_visibility.get()
            || image.color.is_fully_transparent()
            || image.image.id() == TRANSPARENT_IMAGE_HANDLE.id()
            || image.image_mode.uses_slices()
            || uinode.is_empty()
        {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(camera) else {
            continue;
        };

        let atlas_rect = image
            .texture_atlas
            .as_ref()
            .and_then(|s| s.texture_rect(&texture_atlases))
            .map(|r| r.as_rect());

        let mut rect = match (atlas_rect, image.rect) {
            (None, None) => Rect {
                min: Vec2::ZERO,
                max: uinode.size,
            },
            (None, Some(image_rect)) => image_rect,
            (Some(atlas_rect), None) => atlas_rect,
            (Some(atlas_rect), Some(mut image_rect)) => {
                image_rect.min += atlas_rect.min;
                image_rect.max += atlas_rect.min;
                image_rect
            }
        };

        let atlas_scaling = if atlas_rect.is_some() || image.rect.is_some() {
            let atlas_scaling = uinode.size() / rect.size();
            rect.min *= atlas_scaling;
            rect.max *= atlas_scaling;
            Some(atlas_scaling)
        } else {
            None
        };

        extracted_uinodes.uinodes.push(ExtractedUiNode {
            render_entity: commands.spawn(TemporaryRenderEntity).id(),
            stack_index: uinode.stack_index,
            color: image.color.into(),
            rect,
            clip: clip.map(|clip| clip.clip),
            image: image.image.id(),
            extracted_camera_entity,
            item: ExtractedUiItem::Node {
                atlas_scaling,
                transform: transform.compute_matrix(),
                flip_x: image.flip_x,
                flip_y: image.flip_y,
                border: uinode.border,
                border_radius: uinode.border_radius,
                node_type: NodeType::Rect,
            },
            main_entity: entity.into(),
        });
    }
}

pub fn extract_uinode_borders(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    uinode_query: Extract<
        Query<(
            Entity,
            &Node,
            &ComputedNode,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            &ComputedNodeTarget,
            AnyOf<(&BorderColor, &Outline)>,
        )>,
    >,
    camera_map: Extract<UiCameraMap>,
) {
    let image = AssetId::<Image>::default();
    let mut camera_mapper = camera_map.get_mapper();

    for (
        entity,
        node,
        computed_node,
        global_transform,
        inherited_visibility,
        maybe_clip,
        camera,
        (maybe_border_color, maybe_outline),
    ) in &uinode_query
    {
        // Skip invisible borders and removed nodes
        if !inherited_visibility.get() || node.display == Display::None {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(camera) else {
            continue;
        };

        // Don't extract borders with zero width along all edges
        if computed_node.border() != BorderRect::ZERO {
            if let Some(border_color) = maybe_border_color.filter(|bc| !bc.0.is_fully_transparent())
            {
                extracted_uinodes.uinodes.push(ExtractedUiNode {
                    stack_index: computed_node.stack_index,
                    color: border_color.0.into(),
                    rect: Rect {
                        max: computed_node.size(),
                        ..Default::default()
                    },
                    image,
                    clip: maybe_clip.map(|clip| clip.clip),
                    extracted_camera_entity,
                    item: ExtractedUiItem::Node {
                        atlas_scaling: None,
                        transform: global_transform.compute_matrix(),
                        flip_x: false,
                        flip_y: false,
                        border: computed_node.border(),
                        border_radius: computed_node.border_radius(),
                        node_type: NodeType::Border,
                    },
                    main_entity: entity.into(),
                    render_entity: commands.spawn(TemporaryRenderEntity).id(),
                });
            }
        }

        if computed_node.outline_width() <= 0. {
            continue;
        }

        if let Some(outline) = maybe_outline.filter(|outline| !outline.color.is_fully_transparent())
        {
            let outline_size = computed_node.outlined_node_size();
            extracted_uinodes.uinodes.push(ExtractedUiNode {
                render_entity: commands.spawn(TemporaryRenderEntity).id(),
                stack_index: computed_node.stack_index,
                color: outline.color.into(),
                rect: Rect {
                    max: outline_size,
                    ..Default::default()
                },
                image,
                clip: maybe_clip.map(|clip| clip.clip),
                extracted_camera_entity,
                item: ExtractedUiItem::Node {
                    transform: global_transform.compute_matrix(),
                    atlas_scaling: None,
                    flip_x: false,
                    flip_y: false,
                    border: BorderRect::all(computed_node.outline_width()),
                    border_radius: computed_node.outline_radius(),
                    node_type: NodeType::Border,
                },
                main_entity: entity.into(),
            });
        }
    }
}

/// The UI camera is "moved back" by this many units (plus the [`UI_CAMERA_TRANSFORM_OFFSET`]) and also has a view
/// distance of this many units. This ensures that with a left-handed projection,
/// as ui elements are "stacked on top of each other", they are within the camera's view
/// and have room to grow.
// TODO: Consider computing this value at runtime based on the maximum z-value.
const UI_CAMERA_FAR: f32 = 1000.0;

// This value is subtracted from the far distance for the camera's z-position to ensure nodes at z == 0.0 are rendered
// TODO: Evaluate if we still need this.
const UI_CAMERA_TRANSFORM_OFFSET: f32 = -0.1;

/// The ID of the subview associated with a camera on which UI is to be drawn.
///
/// When UI is present, cameras extract to two views: the main 2D/3D one and a
/// UI one. The main 2D or 3D camera gets subview 0, and the corresponding UI
/// camera gets this subview, 1.
const UI_CAMERA_SUBVIEW: u32 = 1;

/// A render-world component that lives on the main render target view and
/// specifies the corresponding UI view.
///
/// For example, if UI is being rendered to a 3D camera, this component lives on
/// the 3D camera and contains the entity corresponding to the UI view.
#[derive(Component)]
/// Entity id of the temporary render entity with the corresponding extracted UI view.
pub struct UiCameraView(pub Entity);

/// A render-world component that lives on the UI view and specifies the
/// corresponding main render target view.
///
/// For example, if the UI is being rendered to a 3D camera, this component
/// lives on the UI view and contains the entity corresponding to the 3D camera.
///
/// This is the inverse of [`UiCameraView`].
#[derive(Component)]
pub struct UiViewTarget(pub Entity);

/// Extracts all UI elements associated with a camera into the render world.
pub fn extract_ui_camera_view(
    mut commands: Commands,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    query: Extract<
        Query<
            (
                Entity,
                RenderEntity,
                &Camera,
                Option<&UiAntiAlias>,
                Option<&BoxShadowSamples>,
            ),
            Or<(With<Camera2d>, With<Camera3d>)>,
        >,
    >,
    mut live_entities: Local<HashSet<RetainedViewEntity>>,
) {
    live_entities.clear();

    for (main_entity, render_entity, camera, ui_anti_alias, shadow_samples) in &query {
        // ignore inactive cameras
        if !camera.is_active {
            commands
                .get_entity(render_entity)
                .expect("Camera entity wasn't synced.")
                .remove::<(UiCameraView, UiAntiAlias, BoxShadowSamples)>();
            continue;
        }

        if let Some(physical_viewport_rect) = camera.physical_viewport_rect() {
            // use a projection matrix with the origin in the top left instead of the bottom left that comes with OrthographicProjection
            let projection_matrix = Mat4::orthographic_rh(
                0.0,
                physical_viewport_rect.width() as f32,
                physical_viewport_rect.height() as f32,
                0.0,
                0.0,
                UI_CAMERA_FAR,
            );
            // We use `UI_CAMERA_SUBVIEW` here so as not to conflict with the
            // main 3D or 2D camera, which will have subview index 0.
            let retained_view_entity =
                RetainedViewEntity::new(main_entity.into(), None, UI_CAMERA_SUBVIEW);
            // Creates the UI view.
            let ui_camera_view = commands
                .spawn((
                    ExtractedView {
                        retained_view_entity,
                        clip_from_view: projection_matrix,
                        world_from_view: GlobalTransform::from_xyz(
                            0.0,
                            0.0,
                            UI_CAMERA_FAR + UI_CAMERA_TRANSFORM_OFFSET,
                        ),
                        clip_from_world: None,
                        hdr: camera.hdr,
                        viewport: UVec4::from((
                            physical_viewport_rect.min,
                            physical_viewport_rect.size(),
                        )),
                        color_grading: Default::default(),
                    },
                    // Link to the main camera view.
                    UiViewTarget(render_entity),
                    TemporaryRenderEntity,
                ))
                .id();

            let mut entity_commands = commands
                .get_entity(render_entity)
                .expect("Camera entity wasn't synced.");
            // Link from the main 2D/3D camera view to the UI view.
            entity_commands.insert(UiCameraView(ui_camera_view));
            if let Some(ui_anti_alias) = ui_anti_alias {
                entity_commands.insert(*ui_anti_alias);
            }
            if let Some(shadow_samples) = shadow_samples {
                entity_commands.insert(*shadow_samples);
            }
            transparent_render_phases.insert_or_clear(retained_view_entity);

            live_entities.insert(retained_view_entity);
        }
    }

    transparent_render_phases.retain(|entity, _| live_entities.contains(entity));
}

pub fn extract_text_sections(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            &ComputedNodeTarget,
            &ComputedTextBlock,
            &TextLayoutInfo,
        )>,
    >,
    text_styles: Extract<Query<&TextColor>>,
    camera_map: Extract<UiCameraMap>,
) {
    let mut start = extracted_uinodes.glyphs.len();
    let mut end = start + 1;

    let mut camera_mapper = camera_map.get_mapper();
    for (
        entity,
        uinode,
        global_transform,
        inherited_visibility,
        clip,
        camera,
        computed_block,
        text_layout_info,
    ) in &uinode_query
    {
        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !inherited_visibility.get() || uinode.is_empty() {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(camera) else {
            continue;
        };

        let transform = global_transform.affine()
            * bevy_math::Affine3A::from_translation((-0.5 * uinode.size()).extend(0.));

        for (
            i,
            PositionedGlyph {
                position,
                atlas_info,
                span_index,
                ..
            },
        ) in text_layout_info.glyphs.iter().enumerate()
        {
            let rect = texture_atlases
                .get(&atlas_info.texture_atlas)
                .unwrap()
                .textures[atlas_info.location.glyph_index]
                .as_rect();
            extracted_uinodes.glyphs.push(ExtractedGlyph {
                transform: transform * Mat4::from_translation(position.extend(0.)),
                rect,
            });

            if text_layout_info.glyphs.get(i + 1).is_none_or(|info| {
                info.span_index != *span_index || info.atlas_info.texture != atlas_info.texture
            }) {
                let color = text_styles
                    .get(
                        computed_block
                            .entities()
                            .get(*span_index)
                            .map(|t| t.entity)
                            .unwrap_or(Entity::PLACEHOLDER),
                    )
                    .map(|text_color| LinearRgba::from(text_color.0))
                    .unwrap_or_default();
                extracted_uinodes.uinodes.push(ExtractedUiNode {
                    render_entity: commands.spawn(TemporaryRenderEntity).id(),
                    stack_index: uinode.stack_index,
                    color,
                    image: atlas_info.texture.id(),
                    clip: clip.map(|clip| clip.clip),
                    extracted_camera_entity,
                    rect,
                    item: ExtractedUiItem::Glyphs { range: start..end },
                    main_entity: entity.into(),
                });
                start = end;
            }

            end += 1;
        }
    }
}

pub fn extract_text_shadows(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    default_ui_camera: Extract<DefaultUiCamera>,
    texture_atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            Option<&UiTargetCamera>,
            &TextLayoutInfo,
            &TextShadow,
        )>,
    >,
    mapping: Extract<Query<RenderEntity>>,
) {
    let mut start = extracted_uinodes.glyphs.len();
    let mut end = start + 1;

    let default_ui_camera = default_ui_camera.get();
    for (
        entity,
        uinode,
        global_transform,
        inherited_visibility,
        clip,
        camera,
        text_layout_info,
        shadow,
    ) in &uinode_query
    {
        let Some(camera_entity) = camera.map(UiTargetCamera::entity).or(default_ui_camera) else {
            continue;
        };

        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !inherited_visibility.get() || uinode.is_empty() {
            continue;
        }

        let Ok(extracted_camera_entity) = mapping.get(camera_entity) else {
            continue;
        };

        let transform = global_transform.affine()
            * Mat4::from_translation(
                (-0.5 * uinode.size() + shadow.offset / uinode.inverse_scale_factor()).extend(0.),
            );

        for (
            i,
            PositionedGlyph {
                position,
                atlas_info,
                span_index,
                ..
            },
        ) in text_layout_info.glyphs.iter().enumerate()
        {
            let rect = texture_atlases
                .get(&atlas_info.texture_atlas)
                .unwrap()
                .textures[atlas_info.location.glyph_index]
                .as_rect();
            extracted_uinodes.glyphs.push(ExtractedGlyph {
                transform: transform * Mat4::from_translation(position.extend(0.)),
                rect,
            });

            if text_layout_info.glyphs.get(i + 1).is_none_or(|info| {
                info.span_index != *span_index || info.atlas_info.texture != atlas_info.texture
            }) {
                extracted_uinodes.uinodes.push(ExtractedUiNode {
                    render_entity: commands.spawn(TemporaryRenderEntity).id(),
                    stack_index: uinode.stack_index,
                    color: shadow.color.into(),
                    image: atlas_info.texture.id(),
                    clip: clip.map(|clip| clip.clip),
                    extracted_camera_entity,
                    rect,
                    item: ExtractedUiItem::Glyphs { range: start..end },
                    main_entity: entity.into(),
                });
                start = end;
            }

            end += 1;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct UiVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    /// Shader flags to determine how to render the UI node.
    /// See [`shader_flags`] for possible values.
    pub flags: u32,
    /// Border radius of the UI node.
    /// Ordering: top left, top right, bottom right, bottom left.
    pub radius: [f32; 4],
    /// Border thickness of the UI node.
    /// Ordering: left, top, right, bottom.
    pub border: [f32; 4],
    /// Size of the UI node.
    pub size: [f32; 2],
    /// Position relative to the center of the UI node.
    pub point: [f32; 2],
}

#[derive(Resource)]
pub struct UiMeta {
    vertices: RawBufferVec<UiVertex>,
    indices: RawBufferVec<u32>,
    view_bind_group: Option<BindGroup>,
}

impl Default for UiMeta {
    fn default() -> Self {
        Self {
            vertices: RawBufferVec::new(BufferUsages::VERTEX),
            indices: RawBufferVec::new(BufferUsages::INDEX),
            view_bind_group: None,
        }
    }
}

pub(crate) const QUAD_VERTEX_POSITIONS: [Vec3; 4] = [
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
    Vec3::new(-0.5, 0.5, 0.0),
];

pub(crate) const QUAD_INDICES: [usize; 6] = [0, 2, 3, 0, 1, 2];

#[derive(Component)]
pub struct UiBatch {
    pub range: Range<u32>,
    pub image: AssetId<Image>,
}

/// The values here should match the values for the constants in `ui.wgsl`
pub mod shader_flags {
    pub const UNTEXTURED: u32 = 0;
    pub const TEXTURED: u32 = 1;
    /// Ordering: top left, top right, bottom right, bottom left.
    pub const CORNERS: [u32; 4] = [0, 2, 2 | 4, 4];
    pub const BORDER: u32 = 8;
}

pub fn queue_uinodes(
    extracted_uinodes: Res<ExtractedUiNodes>,
    ui_pipeline: Res<UiPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<UiPipeline>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    mut render_views: Query<(&UiCameraView, Option<&UiAntiAlias>), With<ExtractedView>>,
    camera_views: Query<&ExtractedView>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<TransparentUi>>,
) {
    let draw_function = draw_functions.read().id::<DrawUi>();
    for (index, extracted_uinode) in extracted_uinodes.uinodes.iter().enumerate() {
        let entity = extracted_uinode.render_entity;
        let Ok((default_camera_view, ui_anti_alias)) =
            render_views.get_mut(extracted_uinode.extracted_camera_entity)
        else {
            continue;
        };

        let Ok(view) = camera_views.get(default_camera_view.0) else {
            continue;
        };

        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let pipeline = pipelines.specialize(
            &pipeline_cache,
            &ui_pipeline,
            UiPipelineKey {
                hdr: view.hdr,
                anti_alias: matches!(ui_anti_alias, None | Some(UiAntiAlias::On)),
            },
        );
        transparent_phase.add(TransparentUi {
            draw_function,
            pipeline,
            entity: (entity, extracted_uinode.main_entity),
            sort_key: (
                FloatOrd(extracted_uinode.stack_index as f32 + stack_z_offsets::NODE),
                entity.index(),
            ),
            index,
            // batch_range will be calculated in prepare_uinodes
            batch_range: 0..0,
            extra_index: PhaseItemExtraIndex::None,
            indexed: true,
        });
    }
}

#[derive(Resource, Default)]
pub struct ImageNodeBindGroups {
    pub values: HashMap<AssetId<Image>, BindGroup>,
}

pub fn prepare_uinodes(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut ui_meta: ResMut<UiMeta>,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    view_uniforms: Res<ViewUniforms>,
    ui_pipeline: Res<UiPipeline>,
    mut image_bind_groups: ResMut<ImageNodeBindGroups>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    events: Res<SpriteAssetEvents>,
    mut previous_len: Local<usize>,
) {
    // If an image has changed, the GpuImage has (probably) changed
    for event in &events.images {
        match event {
            AssetEvent::Added { .. } |
            AssetEvent::Unused { .. } |
            // Images don't have dependencies
            AssetEvent::LoadedWithDependencies { .. } => {}
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                image_bind_groups.values.remove(id);
            }
        };
    }

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let mut batches: Vec<(Entity, UiBatch)> = Vec::with_capacity(*previous_len);

        ui_meta.vertices.clear();
        ui_meta.indices.clear();
        ui_meta.view_bind_group = Some(render_device.create_bind_group(
            "ui_view_bind_group",
            &ui_pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ));

        // Buffer indexes
        let mut vertices_index = 0;
        let mut indices_index = 0;

        for ui_phase in phases.values_mut() {
            let mut batch_item_index = 0;
            let mut batch_image_handle = AssetId::invalid();

            for item_index in 0..ui_phase.items.len() {
                let item = &mut ui_phase.items[item_index];
                if let Some(extracted_uinode) = extracted_uinodes
                    .uinodes
                    .get(item.index)
                    .filter(|n| item.entity() == n.render_entity)
                {
                    let mut existing_batch = batches.last_mut();

                    if batch_image_handle == AssetId::invalid()
                        || existing_batch.is_none()
                        || (batch_image_handle != AssetId::default()
                            && extracted_uinode.image != AssetId::default()
                            && batch_image_handle != extracted_uinode.image)
                    {
                        if let Some(gpu_image) = gpu_images.get(extracted_uinode.image) {
                            batch_item_index = item_index;
                            batch_image_handle = extracted_uinode.image;

                            let new_batch = UiBatch {
                                range: vertices_index..vertices_index,
                                image: extracted_uinode.image,
                            };

                            batches.push((item.entity(), new_batch));

                            image_bind_groups
                                .values
                                .entry(batch_image_handle)
                                .or_insert_with(|| {
                                    render_device.create_bind_group(
                                        "ui_material_bind_group",
                                        &ui_pipeline.image_layout,
                                        &BindGroupEntries::sequential((
                                            &gpu_image.texture_view,
                                            &gpu_image.sampler,
                                        )),
                                    )
                                });

                            existing_batch = batches.last_mut();
                        } else {
                            continue;
                        }
                    } else if batch_image_handle == AssetId::default()
                        && extracted_uinode.image != AssetId::default()
                    {
                        if let Some(gpu_image) = gpu_images.get(extracted_uinode.image) {
                            batch_image_handle = extracted_uinode.image;
                            existing_batch.as_mut().unwrap().1.image = extracted_uinode.image;

                            image_bind_groups
                                .values
                                .entry(batch_image_handle)
                                .or_insert_with(|| {
                                    render_device.create_bind_group(
                                        "ui_material_bind_group",
                                        &ui_pipeline.image_layout,
                                        &BindGroupEntries::sequential((
                                            &gpu_image.texture_view,
                                            &gpu_image.sampler,
                                        )),
                                    )
                                });
                        } else {
                            continue;
                        }
                    }
                    match &extracted_uinode.item {
                        ExtractedUiItem::Node {
                            atlas_scaling,
                            flip_x,
                            flip_y,
                            border_radius,
                            border,
                            node_type,
                            transform,
                        } => {
                            let mut flags = if extracted_uinode.image != AssetId::default() {
                                shader_flags::TEXTURED
                            } else {
                                shader_flags::UNTEXTURED
                            };

                            let mut uinode_rect = extracted_uinode.rect;

                            let rect_size = uinode_rect.size().extend(1.0);

                            // Specify the corners of the node
                            let positions = QUAD_VERTEX_POSITIONS
                                .map(|pos| (*transform * (pos * rect_size).extend(1.)).xyz());
                            let points = QUAD_VERTEX_POSITIONS.map(|pos| pos.xy() * rect_size.xy());

                            // Calculate the effect of clipping
                            // Note: this won't work with rotation/scaling, but that's much more complex (may need more that 2 quads)
                            let mut positions_diff = if let Some(clip) = extracted_uinode.clip {
                                [
                                    Vec2::new(
                                        f32::max(clip.min.x - positions[0].x, 0.),
                                        f32::max(clip.min.y - positions[0].y, 0.),
                                    ),
                                    Vec2::new(
                                        f32::min(clip.max.x - positions[1].x, 0.),
                                        f32::max(clip.min.y - positions[1].y, 0.),
                                    ),
                                    Vec2::new(
                                        f32::min(clip.max.x - positions[2].x, 0.),
                                        f32::min(clip.max.y - positions[2].y, 0.),
                                    ),
                                    Vec2::new(
                                        f32::max(clip.min.x - positions[3].x, 0.),
                                        f32::min(clip.max.y - positions[3].y, 0.),
                                    ),
                                ]
                            } else {
                                [Vec2::ZERO; 4]
                            };

                            let positions_clipped = [
                                positions[0] + positions_diff[0].extend(0.),
                                positions[1] + positions_diff[1].extend(0.),
                                positions[2] + positions_diff[2].extend(0.),
                                positions[3] + positions_diff[3].extend(0.),
                            ];

                            let points = [
                                points[0] + positions_diff[0],
                                points[1] + positions_diff[1],
                                points[2] + positions_diff[2],
                                points[3] + positions_diff[3],
                            ];

                            let transformed_rect_size = transform.transform_vector3(rect_size);

                            // Don't try to cull nodes that have a rotation
                            // In a rotation around the Z-axis, this value is 0.0 for an angle of 0.0 or π
                            // In those two cases, the culling check can proceed normally as corners will be on
                            // horizontal / vertical lines
                            // For all other angles, bypass the culling check
                            // This does not properly handles all rotations on all axis
                            if transform.x_axis[1] == 0.0 {
                                // Cull nodes that are completely clipped
                                if positions_diff[0].x - positions_diff[1].x
                                    >= transformed_rect_size.x
                                    || positions_diff[1].y - positions_diff[2].y
                                        >= transformed_rect_size.y
                                {
                                    continue;
                                }
                            }
                            let uvs = if flags == shader_flags::UNTEXTURED {
                                [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]
                            } else {
                                let image = gpu_images.get(extracted_uinode.image).expect(
                                    "Image was checked during batching and should still exist",
                                );
                                // Rescale atlases. This is done here because we need texture data that might not be available in Extract.
                                let atlas_extent = atlas_scaling
                                    .map(|scaling| image.size_2d().as_vec2() * scaling)
                                    .unwrap_or(uinode_rect.max);
                                if *flip_x {
                                    core::mem::swap(&mut uinode_rect.max.x, &mut uinode_rect.min.x);
                                    positions_diff[0].x *= -1.;
                                    positions_diff[1].x *= -1.;
                                    positions_diff[2].x *= -1.;
                                    positions_diff[3].x *= -1.;
                                }
                                if *flip_y {
                                    core::mem::swap(&mut uinode_rect.max.y, &mut uinode_rect.min.y);
                                    positions_diff[0].y *= -1.;
                                    positions_diff[1].y *= -1.;
                                    positions_diff[2].y *= -1.;
                                    positions_diff[3].y *= -1.;
                                }
                                [
                                    Vec2::new(
                                        uinode_rect.min.x + positions_diff[0].x,
                                        uinode_rect.min.y + positions_diff[0].y,
                                    ),
                                    Vec2::new(
                                        uinode_rect.max.x + positions_diff[1].x,
                                        uinode_rect.min.y + positions_diff[1].y,
                                    ),
                                    Vec2::new(
                                        uinode_rect.max.x + positions_diff[2].x,
                                        uinode_rect.max.y + positions_diff[2].y,
                                    ),
                                    Vec2::new(
                                        uinode_rect.min.x + positions_diff[3].x,
                                        uinode_rect.max.y + positions_diff[3].y,
                                    ),
                                ]
                                .map(|pos| pos / atlas_extent)
                            };

                            let color = extracted_uinode.color.to_f32_array();
                            if *node_type == NodeType::Border {
                                flags |= shader_flags::BORDER;
                            }

                            for i in 0..4 {
                                ui_meta.vertices.push(UiVertex {
                                    position: positions_clipped[i].into(),
                                    uv: uvs[i].into(),
                                    color,
                                    flags: flags | shader_flags::CORNERS[i],
                                    radius: [
                                        border_radius.top_left,
                                        border_radius.top_right,
                                        border_radius.bottom_right,
                                        border_radius.bottom_left,
                                    ],
                                    border: [border.left, border.top, border.right, border.bottom],
                                    size: rect_size.xy().into(),
                                    point: points[i].into(),
                                });
                            }

                            for &i in &QUAD_INDICES {
                                ui_meta.indices.push(indices_index + i as u32);
                            }

                            vertices_index += 6;
                            indices_index += 4;
                        }
                        ExtractedUiItem::Glyphs { range } => {
                            let image = gpu_images
                                .get(extracted_uinode.image)
                                .expect("Image was checked during batching and should still exist");

                            let atlas_extent = image.size_2d().as_vec2();

                            let color = extracted_uinode.color.to_f32_array();
                            for glyph in &extracted_uinodes.glyphs[range.clone()] {
                                let glyph_rect = glyph.rect;
                                let size = glyph.rect.size();

                                let rect_size = glyph_rect.size().extend(1.0);

                                // Specify the corners of the glyph
                                let positions = QUAD_VERTEX_POSITIONS.map(|pos| {
                                    (glyph.transform * (pos * rect_size).extend(1.)).xyz()
                                });

                                let positions_diff = if let Some(clip) = extracted_uinode.clip {
                                    [
                                        Vec2::new(
                                            f32::max(clip.min.x - positions[0].x, 0.),
                                            f32::max(clip.min.y - positions[0].y, 0.),
                                        ),
                                        Vec2::new(
                                            f32::min(clip.max.x - positions[1].x, 0.),
                                            f32::max(clip.min.y - positions[1].y, 0.),
                                        ),
                                        Vec2::new(
                                            f32::min(clip.max.x - positions[2].x, 0.),
                                            f32::min(clip.max.y - positions[2].y, 0.),
                                        ),
                                        Vec2::new(
                                            f32::max(clip.min.x - positions[3].x, 0.),
                                            f32::min(clip.max.y - positions[3].y, 0.),
                                        ),
                                    ]
                                } else {
                                    [Vec2::ZERO; 4]
                                };

                                let positions_clipped = [
                                    positions[0] + positions_diff[0].extend(0.),
                                    positions[1] + positions_diff[1].extend(0.),
                                    positions[2] + positions_diff[2].extend(0.),
                                    positions[3] + positions_diff[3].extend(0.),
                                ];

                                // cull nodes that are completely clipped
                                let transformed_rect_size =
                                    glyph.transform.transform_vector3(rect_size);
                                if positions_diff[0].x - positions_diff[1].x
                                    >= transformed_rect_size.x.abs()
                                    || positions_diff[1].y - positions_diff[2].y
                                        >= transformed_rect_size.y.abs()
                                {
                                    continue;
                                }

                                let uvs = [
                                    Vec2::new(
                                        glyph.rect.min.x + positions_diff[0].x,
                                        glyph.rect.min.y + positions_diff[0].y,
                                    ),
                                    Vec2::new(
                                        glyph.rect.max.x + positions_diff[1].x,
                                        glyph.rect.min.y + positions_diff[1].y,
                                    ),
                                    Vec2::new(
                                        glyph.rect.max.x + positions_diff[2].x,
                                        glyph.rect.max.y + positions_diff[2].y,
                                    ),
                                    Vec2::new(
                                        glyph.rect.min.x + positions_diff[3].x,
                                        glyph.rect.max.y + positions_diff[3].y,
                                    ),
                                ]
                                .map(|pos| pos / atlas_extent);

                                for i in 0..4 {
                                    ui_meta.vertices.push(UiVertex {
                                        position: positions_clipped[i].into(),
                                        uv: uvs[i].into(),
                                        color,
                                        flags: shader_flags::TEXTURED | shader_flags::CORNERS[i],
                                        radius: [0.0; 4],
                                        border: [0.0; 4],
                                        size: size.into(),
                                        point: [0.0; 2],
                                    });
                                }

                                for &i in &QUAD_INDICES {
                                    ui_meta.indices.push(indices_index + i as u32);
                                }

                                vertices_index += 6;
                                indices_index += 4;
                            }
                        }
                    }
                    existing_batch.unwrap().1.range.end = vertices_index;
                    ui_phase.items[batch_item_index].batch_range_mut().end += 1;
                } else {
                    batch_image_handle = AssetId::invalid();
                }
            }
        }

        ui_meta.vertices.write_buffer(&render_device, &render_queue);
        ui_meta.indices.write_buffer(&render_device, &render_queue);
        *previous_len = batches.len();
        commands.insert_or_spawn_batch(batches);
    }
    extracted_uinodes.clear();
}
