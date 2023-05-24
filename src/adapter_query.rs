#[non_exhaustive]
pub struct AdapterQuery<'a> {
    pub compatible_surface: Option<&'a wgpu::Surface>,
    pub physical_blacklist: &'a [&'a wgpu::Adapter],
    pub force_adapter_type: Option<wgpu::DeviceType>,
}

impl<'a> Default for AdapterQuery<'a> {
    fn default() -> Self {
        Self {
            compatible_surface: None,
            physical_blacklist: &[],
            force_adapter_type: None,
        }
    }
}

impl<'a> AdapterQuery<'a> {
    pub fn is_adapter_allowed(&self, adapter: &wgpu::Adapter) -> bool {
        if let Some(forced_type) = self.force_adapter_type {
            if forced_type != adapter.get_info().device_type {
                return false;
            }
        }

        if let Some(surface) = self.compatible_surface {
            if !adapter.is_surface_supported(surface) {
                return false;
            }
        }

        for device in self.physical_blacklist.iter() {
            if adapter.get_info().device == device.get_info().device {
                return false;
            }
        }

        return true;
    }
}

fn device_type_ranking(dt: &wgpu::DeviceType) -> usize {
    match dt {
        wgpu::DeviceType::DiscreteGpu => 4,
        wgpu::DeviceType::IntegratedGpu => 3,
        wgpu::DeviceType::VirtualGpu => 2,
        wgpu::DeviceType::Other => 1, // If we can run graphics on it, it's probably better than plain CPU
        wgpu::DeviceType::Cpu => 0,
    }
}

macro_rules! lim_cmp {
    (fn $fn_name:ident = { $fst:ident $( =|> $nxt:ident)* }) => {
        fn $fn_name(lim1: &wgpu::Limits, lim2: &wgpu::Limits) -> std::cmp::Ordering {
            lim1.$fst.cmp(&lim2.$fst)
            $(.then(
                lim1.$nxt.cmp(&lim2.$nxt)
            ))*
        }
    };
}

// Gives the order of comparisons to decide a best device to use
lim_cmp!(fn compare_limits = {
    max_buffer_size
    =|> max_compute_workgroup_storage_size
    =|> max_push_constant_size
    =|> max_texture_dimension_2d
    =|> max_texture_dimension_1d
    =|> max_texture_dimension_3d
    }
);

// We're only adding methods to wgpu::Instance
mod sealed {
    pub trait SealedInstance {}

    impl SealedInstance for wgpu::Instance {}
}

pub(crate) fn request_powerful_adapter<'a>(
    instance: &wgpu::Instance,
    backends: wgpu::Backends,
    query: AdapterQuery<'a>,
) -> Option<wgpu::Adapter> {
    let all: Vec<wgpu::Adapter> = instance
        .enumerate_adapters(backends)
        .filter(|adapter: &wgpu::Adapter| {
            // Check surface support
            query.is_adapter_allowed(adapter)
        })
        .collect();

    // Get best device category
    let best_device_type: wgpu::DeviceType = all
        .iter()
        .map(|a: &wgpu::Adapter| a.get_info().device_type)
        .max_by_key(device_type_ranking)?;

    return all
        .into_iter()
        .filter(|adapter: &wgpu::Adapter| adapter.get_info().device_type == best_device_type)
        // Get best by limits
        .max_by(|adapter1: &wgpu::Adapter, adapter2: &wgpu::Adapter| {
            compare_limits(&adapter1.limits(), &adapter2.limits())
        });
}
