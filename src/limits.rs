//! Provides some more wgpu::Limits functionality, like taking the element-wise minimum or maximum of two
//! limits.

use std::cmp::{max, min};

macro_rules! binop_limits {
    (@forall ($op_on_max:ident, $op_on_min:ident)($a:ident, $b:ident) max{$($max_field:ident),* $(,)?} min{$($min_field:ident),* $(,)?}) => {
        wgpu::Limits{
            $(
                $max_field : $op_on_max($a.$max_field, $b.$max_field),
            )*
            $(
                $min_field : $op_on_min($a.$min_field, $b.$min_field),
            )*
        }
    };

    (($op_on_max:ident, $op_on_min:ident)($a:ident, $b:ident)) => {
        binop_limits!(@forall
            ($op_on_max, $op_on_min)($a, $b)
            max {
                max_texture_dimension_1d,
                max_texture_dimension_2d,
                max_texture_dimension_3d,
                max_texture_array_layers,
                max_bind_groups,
                max_bindings_per_bind_group,
                max_dynamic_uniform_buffers_per_pipeline_layout,
                max_dynamic_storage_buffers_per_pipeline_layout,
                max_sampled_textures_per_shader_stage,
                max_samplers_per_shader_stage,
                max_storage_buffers_per_shader_stage,
                max_storage_textures_per_shader_stage,
                max_uniform_buffers_per_shader_stage,
                max_uniform_buffer_binding_size,
                max_storage_buffer_binding_size,
                max_vertex_buffers,
                max_buffer_size,
                max_vertex_attributes,
                max_vertex_buffer_array_stride,
                max_inter_stage_shader_components,
                max_compute_workgroup_storage_size,
                max_compute_invocations_per_workgroup,
                max_compute_workgroup_size_x,
                max_compute_workgroup_size_y,
                max_compute_workgroup_size_z,
                max_compute_workgroups_per_dimension,
                max_push_constant_size,
            }
            min {
                min_uniform_buffer_offset_alignment,
                min_storage_buffer_offset_alignment,
            }
        )
    };
}

pub(crate) fn limits_intersection(lhs: &wgpu::Limits, rhs: &wgpu::Limits) -> wgpu::Limits {
    binop_limits!((min, max)(lhs, rhs))
}

pub(crate) fn limits_union(lhs: &wgpu::Limits, rhs: &wgpu::Limits) -> wgpu::Limits {
    binop_limits!((max, min)(lhs, rhs))
}
