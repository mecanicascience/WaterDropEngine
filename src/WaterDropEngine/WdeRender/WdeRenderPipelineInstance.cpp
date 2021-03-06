#include "WdeRenderPipelineInstance.hpp"
#include "../WaterDropEngine.hpp"

namespace wde::render {
	WdeRenderPipelineInstance::~WdeRenderPipelineInstance() {
		WDE_PROFILE_FUNCTION();
		// Destroy render passes
		_passes.clear();
	}


	void WdeRenderPipelineInstance::tick() {
		WDE_PROFILE_FUNCTION();
		render::CoreInstance& renderer = WaterDropEngine::get().getRender().getInstance();
		logger::log(LogLevel::DEBUG, LogChannel::RENDER) << "Drawing next frame to the screen with id " << renderer.getCurrentFrame() << "." << logger::endl;

		// Acquire next image from swapchain and signal it to semaphore _imageAvailableSemaphores
		logger::log(LogLevel::DEBUG, LogChannel::RENDER) << "Acquiring swapchain next frame." << logger::endl;
		{
			WDE_PROFILE_SCOPE("wde::render::WdeRenderPipelineInstance::tick()::acquireImage");
			renderer.getSwapchain().aquireNextImage();
		}

		// Acquire command buffer
		logger::log(LogLevel::DEBUG, LogChannel::RENDER) << "Acquiring and preparing frame command buffer." << logger::endl;
		render::CommandBuffer& commandBuffer = *renderer.getCommandBuffers()[renderer.getCurrentFrame()];
		if (!commandBuffer.isRunning())
			commandBuffer.begin();

		// Engine recording commands to the current frame command buffer
		{
			WDE_PROFILE_SCOPE("wde::render::WdeRenderPipelineInstance::tick()::render");

			// ==== RENDER COMMANDS ====
			auto scene = WaterDropEngine::get().getInstance().getScene();
			render(commandBuffer, *scene);
		}

		// Wait for last swapchain image to finish rendering before sending to queue
		logger::log(LogLevel::DEBUG, LogChannel::RENDER) << "Waiting for last swapchain fence to end presentation." << logger::endl;
		{
			WDE_PROFILE_SCOPE("wde::render::WdeRenderPipelineInstance::tick()::vkWaitForFences");
			vkWaitForFences(renderer.getDevice().getDevice(), 1, &renderer.getSwapchain().getInFlightFences()[(renderer.getSwapchain().getActiveImageIndex() - 1) % renderer.getMaxFramesInFlight()], VK_TRUE, UINT64_MAX);
		}

		// Submit command buffer
		logger::log(LogLevel::DEBUG, LogChannel::RENDER) << "Submitting command buffer to graphics queue." << logger::endl;
		{
			WDE_PROFILE_SCOPE("wde::render::WdeRenderPipelineInstance::tick()::submitCommandBuffer");
			commandBuffer.end();
			commandBuffer.submit(
					renderer.getSwapchain().getInFlightFences()[renderer.getSwapchain().getActiveImageIndex()], // When submitting is done, signal it to the fence
					renderer.getSwapchain().getImageAvailableSemaphores()[renderer.getSwapchain().getActiveImageIndex()], // Wait for swapchain to acquire image
					renderer.getSwapchain().getRenderFinishedSemaphores()[renderer.getSwapchain().getActiveImageIndex()]); // Say that image has been presented
		}

		// Send the current swapchain image to the presentation device queue
		logger::log(LogLevel::DEBUG, LogChannel::RENDER) << "Sending swapchain frame to presentation queue." << logger::endl;
		{
			WDE_PROFILE_SCOPE("wde::render::WdeRenderPipelineInstance::tick()::present");
			auto presentResult = renderer.getSwapchain().presentToQueue(renderer.getDevice().getPresentQueue());
			if (presentResult != VK_SUCCESS)
				throw WdeException(wde::LogChannel::RENDER, "Failed to present swap chain image.");
		}

		// Increase rendered frame ID
		renderer.getCurrentFrame() = (renderer.getCurrentFrame() + 1) % renderer.getMaxFramesInFlight();
	}

	void WdeRenderPipelineInstance::onWindowResized() {
		WDE_PROFILE_FUNCTION();
		// Recreate render passes
		_passes.clear();
		setStructure(_structure);
	}



	// Pass command manager
	void WdeRenderPipelineInstance::setStructure(const std::vector<RenderPassStructure> &structure) {
		WDE_PROFILE_FUNCTION();
		_structure = structure;

		// Check if attachments setup
		if (_attachments.empty())
			throw WdeException(LogChannel::RENDER, "Tried to create render passes before creating attachments in the render pipeline.");

		// Create passes
		uint32_t iterator = 0;
		for (auto& str : structure) {
			// Check if passes IDs are in order
			if (iterator != str._passID)
				throw WdeException(LogChannel::RENDER, "Missing render pass with ID = " + std::to_string(iterator) + ".");

			// Check if subpasses IDs are in order
			uint32_t iterator2 = 0;
			for (auto& sub : str._subPasses) {
				if (iterator2 != sub._subpassID)
					throw WdeException(LogChannel::RENDER,
					                   "Missing render subpass with ID = " + std::to_string(iterator2) +
					                   " in render pass with ID = " + std::to_string(iterator) + ".");
				iterator2++;
			}

			// Create subpass
			_passes.push_back(std::make_unique<RenderPass>(_attachments, str._subPasses));
			iterator++;
		}
	}


	// Render passes commands
	void WdeRenderPipelineInstance::beginRenderPass(uint32_t index) {
		WDE_PROFILE_FUNCTION();
		if (_currentRenderPassID != -1)
			throw WdeException(LogChannel::RENDER, "Trying to begin pass " + std::to_string(index) + " while pass " + std::to_string(_currentRenderPassID) + " has already began.");
		if (index >= _passes.size())
			throw WdeException(LogChannel::RENDER, "Trying to begin pass " + std::to_string(index) + " which wasn't created.");

		_currentRenderPassID = index;
		_passes[_currentRenderPassID]->start();
	}

	void WdeRenderPipelineInstance::endRenderPass()  {
		WDE_PROFILE_FUNCTION();
		if (_currentRenderSubPassID != -1)
			throw WdeException(LogChannel::RENDER, "Trying to end render pass " + std::to_string(_currentRenderPassID) + " while subpass " + std::to_string(_currentRenderSubPassID) + " has already began.");

		_passes[_currentRenderPassID]->end();
		_currentRenderPassID = -1;
	}

	void WdeRenderPipelineInstance::beginRenderSubPass(uint32_t index) {
		WDE_PROFILE_FUNCTION();
		if (_currentRenderPassID == -1)
			throw WdeException(LogChannel::RENDER, "Trying to begin subpass " + std::to_string(index) + " outside of a render pass.");
		if (_currentRenderSubPassID != -1)
			throw WdeException(LogChannel::RENDER, "Trying to begin subpass " + std::to_string(index) + " while subpass " + std::to_string(_currentRenderSubPassID)
			                                       + " has already began in render pass " + std::to_string(_currentRenderPassID));
		if (index >= _passes[_currentRenderPassID]->getSubPassesCount())
			throw WdeException(LogChannel::RENDER, "Trying to begin pass " + std::to_string(index) + " which wasn't created.");

		_currentRenderSubPassID = index;
		_passes[_currentRenderPassID]->startSubPass(index);
	}

	void WdeRenderPipelineInstance::endRenderSubPass() {
		WDE_PROFILE_FUNCTION();
		if (_currentRenderPassID == -1)
			throw WdeException(LogChannel::RENDER, "Trying to end subpass " + std::to_string(_currentRenderSubPassID) + " outside of a render pass.");

		_passes[_currentRenderPassID]->endSubPass(_currentRenderSubPassID);
		_currentRenderSubPassID = -1;
	}
}

