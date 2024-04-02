mod options;

use std::fmt;

use futures::executor::block_on;
use wgpu::util::DeviceExt as _;

#[derive(Debug)]
pub struct Hasher {
	device: wgpu::Device,
	compute_pipeline: wgpu::ComputePipeline,
	bind_group: wgpu::BindGroup,
	input_header_buffer: wgpu::Buffer,
	input_target_buffer: wgpu::Buffer,
	output_buffer: wgpu::Buffer,
	mappable_buffer: wgpu::Buffer,
	queue: wgpu::Queue,

	_pipeline_layout: wgpu::PipelineLayout,
	_bind_group_layout: wgpu::BindGroupLayout,
	_shader: wgpu::ShaderModule,
}

#[derive(Debug)]
pub enum Error {
	NoAdapter,
	NoDevice,
	BufferAsync(wgpu::BufferAsyncError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::NoAdapter => write!(f, "no adapter found"),
			Self::NoDevice => write!(f, "no device found"),
			Self::BufferAsync(e) => write!(f, "buffer async error: {}", e),
		}
	}
}

impl std::error::Error for Error {}

impl Hasher {
	pub fn new() -> Result<Self, Error> {
		let instance = wgpu::Instance::default();

		let adapter = Self::request_adapter(&instance)?;
		let (device, queue) = Self::request_device(&adapter)?;
		let shader = device.create_shader_module(options::SHADER_DESC);

		let bind_group_layout = device.create_bind_group_layout(&options::BIND_GROUP_LAYOUT);

		let input_header_buffer = device.create_buffer_init(&options::INPUT_HEADER_DESC);
		let input_target_buffer = device.create_buffer_init(&options::INPUT_TARGET_DESC);
		let output_buffer = device.create_buffer(&options::OUTPUT_DESC);
		let mappable_buffer = device.create_buffer(&options::MAPPABLE_DESC);

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Compute Bind Group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &input_header_buffer,
						offset: 0,
						size: None,
					}),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &input_target_buffer,
						offset: 0,
						size: None,
					}),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &output_buffer,
						offset: 0,
						size: None,
					}),
				},
			],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("Compute Pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader,
			entry_point: "main",
		});

		Ok(Self {
			device,
			compute_pipeline,
			bind_group,
			input_header_buffer,
			input_target_buffer,
			output_buffer,
			mappable_buffer,
			queue,
			_pipeline_layout: pipeline_layout,
			_bind_group_layout: bind_group_layout,
			_shader: shader,
		})
	}

	pub fn process(&self, block: [u8; 80], target: [u8; 32]) -> Result<[u8; 80], Error> {
		let command = self.create_command_buffer(block, target);

		self.queue.submit(Some(command));
		self.wait_for_next().map_err(Error::BufferAsync)
	}

	fn wait_for_next(&self) -> Result<[u8; 80], wgpu::BufferAsyncError> {
		let buffer_slice = self.mappable_buffer.slice(..);
		let (tx, rx) = oneshot::channel();

		buffer_slice.map_async(wgpu::MapMode::Read, |res| {
			tx.send(res).unwrap();
		});

		self.device.poll(wgpu::Maintain::Wait);

		rx.recv().unwrap()?;

		let data = buffer_slice.get_mapped_range();

		let block: &[u8] = &data;
		let block = block.try_into().unwrap();

		drop(data);
		self.mappable_buffer.unmap();

		Ok(block)
	}

	fn create_command_buffer(&self, input: [u8; 80], target: [u8; 32]) -> wgpu::CommandBuffer {
		// overwrite the input buffer with the new input
		self.queue
			.write_buffer(&self.input_header_buffer, 0, &input);

		// overwrite the input size buffer with the new input
		self.queue
			.write_buffer(&self.input_target_buffer, 0, &target);

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Compute Encoder"),
			});

		{
			let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
				label: Some("Compute Pass"),
				timestamp_writes: None,
			});

			compute_pass.set_pipeline(&self.compute_pipeline);
			compute_pass.set_bind_group(0, &self.bind_group, &[]);
			// NOTE: when modifying this value, also change `numWorkgroups` in sha256.wgsl
			compute_pass.dispatch_workgroups(32, 1, 1);
		}

		encoder.copy_buffer_to_buffer(&self.output_buffer, 0, &self.mappable_buffer, 0, 80);
		encoder.finish()
	}

	fn request_adapter(instance: &wgpu::Instance) -> Result<wgpu::Adapter, Error> {
		wgpu::Instance::default();
		block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
			.ok_or(Error::NoAdapter)
	}

	fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), Error> {
		let (device, queue) =
			block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
				.map_err(|_| Error::NoDevice)?;

		Ok((device, queue))
	}
}
