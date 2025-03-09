// Copyright (c) 2019-present Dmitry Stepanov and Fyrox Engine contributors.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![warn(missing_docs)]

//! Frame buffer is a set of images that is used as a storage for an image generated by a renderer.
//! It consists of one or more color buffers and an optional depth/stencil buffer. See [`GpuFrameBufferTrait`]
//! docs for more info.

use crate::{
    buffer::GpuBuffer,
    core::{color::Color, math::Rect},
    define_shared_wrapper,
    error::FrameworkError,
    geometry_buffer::GpuGeometryBuffer,
    gpu_program::GpuProgram,
    gpu_texture::{CubeMapFace, GpuTexture},
    DrawParameters, ElementRange,
};
use fyrox_core::define_as_any_trait;

/// Frame buffer attachment kind.
#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug, Eq)]
pub enum AttachmentKind {
    /// Color attachment, it should have a format that supports rendering (for example it cannot be
    /// a compressed texture format).
    Color,
    /// Combined depth + stencil (usually it is 24 bits for depth and 8 for stencil) attachment.
    DepthStencil,
    /// Depth-only attachment. Usually it is 16 or 32 bits texture.
    Depth,
}

/// Frame buffer attachment.
pub struct Attachment {
    /// Current kind of attachment. Tells the renderer how the texture should be used.
    pub kind: AttachmentKind,
    /// A texture that is used to write the rendered image to.
    pub texture: GpuTexture,
}

impl Attachment {
    /// Creates a new [`AttachmentKind::Color`] attachment with the given texture.
    pub fn color(texture: GpuTexture) -> Self {
        Self {
            kind: AttachmentKind::Color,
            texture,
        }
    }

    /// Creates a new [`AttachmentKind::Depth`] attachment with the given texture.
    pub fn depth(texture: GpuTexture) -> Self {
        Self {
            kind: AttachmentKind::Depth,
            texture,
        }
    }

    /// Creates a new [`AttachmentKind::DepthStencil`] attachment with the given texture.
    pub fn depth_stencil(texture: GpuTexture) -> Self {
        Self {
            kind: AttachmentKind::DepthStencil,
            texture,
        }
    }
}

/// Defines a range of data in a particular buffer.
#[derive(Default)]
pub enum BufferDataUsage {
    /// Use everything at once.
    #[default]
    UseEverything,
    /// Use just a segment of data starting from the given `offset` with `size` bytes. It is used
    /// in cases where you have a large buffer with lots of small blocks of information about
    /// different objects. Instead of having a number of small buffers (which is memory- and performance
    /// inefficient), you put everything into a large buffer and fill it lots of info at once and then
    /// binding segments of the data the to the pipeline.
    UseSegment {
        /// Offset from the beginning of the buffer in bytes.
        offset: usize,
        /// Size of the data block in bytes.
        size: usize,
    },
}

/// A resource binding defines where to bind specific GPU resources.
pub enum ResourceBinding {
    /// Texture binding.
    Texture {
        /// A shared reference to a texture.
        texture: GpuTexture,
        /// Binding mode for the texture.
        binding: usize,
    },
    /// Generic data buffer binding.
    Buffer {
        /// A shared reference to a buffer.
        buffer: GpuBuffer,
        /// Binding mode for the buffer.
        binding: usize,
        /// Data portion to use.
        data_usage: BufferDataUsage,
    },
}

impl ResourceBinding {
    /// Creates a new explicit texture binding.
    pub fn texture(texture: &GpuTexture, binding: usize) -> Self {
        Self::Texture {
            texture: texture.clone(),
            binding,
        }
    }

    /// Creates a new explicit buffer binding.
    pub fn buffer(buffer: &GpuBuffer, binding: usize, data_usage: BufferDataUsage) -> Self {
        Self::Buffer {
            buffer: buffer.clone(),
            binding,
            data_usage,
        }
    }
}

/// Resource binding group defines a set of bindings.
pub struct ResourceBindGroup<'a> {
    /// A reference to resource bindings array.
    pub bindings: &'a [ResourceBinding],
}

/// Statistics for a single GPU draw call.
#[derive(Debug, Copy, Clone, Default)]
#[must_use]
pub struct DrawCallStatistics {
    /// Total number of rendered triangles.
    pub triangles: usize,
}

define_as_any_trait!(GpuFrameBufferAsAny => GpuFrameBufferTrait);

/// Frame buffer is a set of images that is used as a storage for an image generated by a renderer.
/// It consists of one or more color buffers and an optional depth/stencil buffer. Frame buffer is
/// a high level abstraction that consolidates multiple images and supports drawing meshes to them
/// with various drawing options.
pub trait GpuFrameBufferTrait: GpuFrameBufferAsAny {
    /// Returns a list of color attachments.
    fn color_attachments(&self) -> &[Attachment];

    /// Returns an optional depth/stencil attachment.
    fn depth_attachment(&self) -> Option<&Attachment>;

    /// Sets an active face of a cube map (only for frame buffers that using cube maps for rendering).
    fn set_cubemap_face(&self, attachment_index: usize, face: CubeMapFace);

    /// Performs data transfer from one frame buffer to another with scaling. It copies a region
    /// defined by `src_x0`, `src_y0`, `src_x1`, `src_y1` coordinates from the frame buffer and
    /// "pastes" it to the other frame buffer into a region defined by `dst_x0`, `dst_y0`, `dst_x1`,
    /// `dst_y1` coordinates. If the source rectangle does not match the destination, the image will
    /// be interpolated using nearest interpolation.
    ///
    /// This method can copy only specific parts of the image: `copy_color` tells the method to copy
    /// the data from
    fn blit_to(
        &self,
        dest: &GpuFrameBuffer,
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        copy_color: bool,
        copy_depth: bool,
        copy_stencil: bool,
    );

    /// Clears the frame buffer in the given viewport with the given set of optional values. This
    /// method clears multiple attachments at once. What will be cleared defined by the provided
    /// values. If `color` is not [`None`], then all the color attachments will be cleared with the
    /// given color. The same applies to depth and stencil buffers.
    fn clear(
        &self,
        viewport: Rect<i32>,
        color: Option<Color>,
        depth: Option<f32>,
        stencil: Option<i32>,
    );

    /// Draws the specified geometry buffer using the given GPU program and a set of resources. This
    /// method the main method to draw anything.
    ///
    /// `geometry` - defines a [`GpuGeometryBuffer`], that contains vertices and index buffers and
    /// essentially defines a mesh to render.
    /// `viewport` - defines an area on screen that will be used to draw.
    /// `program` - a [`GpuProgram`] defines a set of shaders (usually a pair of vertex + fragment)
    /// that will define how the mesh will be rendered.
    /// `params` - [`DrawParameters`] defines the state of graphics pipeline and essentially sets
    /// a bunch of various parameters (such as backface culling, blending mode, various tests, etc.)
    /// that will define how the rendering process is performed.
    /// `resources` - a set of resource bind groups, that in their turn provides a set of resources
    /// that bound to specific binding points.
    /// `element_range` - defines which range of elements to draw.
    fn draw(
        &self,
        geometry: &GpuGeometryBuffer,
        viewport: Rect<i32>,
        program: &GpuProgram,
        params: &DrawParameters,
        resources: &[ResourceBindGroup],
        element_range: ElementRange,
    ) -> Result<DrawCallStatistics, FrameworkError>;

    /// Almost the same as [`Self::draw`], but draws multiple instances at once. The caller must
    /// supply all the required data per each instance, it could be done in different ways. The data
    /// could be supplied in vertex attributes, uniform buffers, textures, etc.
    fn draw_instances(
        &self,
        instance_count: usize,
        geometry: &GpuGeometryBuffer,
        viewport: Rect<i32>,
        program: &GpuProgram,
        params: &DrawParameters,
        resources: &[ResourceBindGroup],
        element_range: ElementRange,
    ) -> Result<DrawCallStatistics, FrameworkError>;
}

define_shared_wrapper!(GpuFrameBuffer<dyn GpuFrameBufferTrait>);
