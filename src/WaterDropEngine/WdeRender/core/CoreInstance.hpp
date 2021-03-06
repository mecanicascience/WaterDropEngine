#pragma once

#include <unordered_set>
#include <map>

#include "../../../wde.hpp"
#include "CoreWindow.hpp"
#include "CoreDevice.hpp"
#include "../render/Swapchain.hpp"
#include "../commands/CommandPool.hpp"
#include "../commands/CommandBuffer.hpp"
#include "../descriptors/DescriptorLayoutCache.hpp"

namespace wde::render {
	class CoreInstance : public NonCopyable {
		public:
			// Core functions
			explicit CoreInstance(CoreWindow& window);
			void cleanUp();
			void start();
			void onWindowResized();


			// Getters and setters
			VkInstance& getInstance() { return _instance; }
			VkSurfaceKHR& getSurface() { return _surface; }
			CoreDevice& getDevice() { return *_device; }
			static bool enableValidationLayers() {
				#ifdef WDE_ENGINE_MODE_PRODUCTION
					return false;
				#endif
				#ifdef WDE_ENGINE_MODE_DEBUG
					return true;
				#endif
			}
			std::vector<const char *>& getValidationLayers() { return _validationLayers; }
			int getMaxFramesInFlight() const { return _currentFramesInFlightCount; }
			void setFramesInFlightCount(int count) { _currentFramesInFlightCount = count; }
			Swapchain& getSwapchain() { return *_swapchain; }
			std::size_t& getCurrentFrame() { return _currentFrame; }
			std::vector<std::unique_ptr<CommandBuffer>>& getCommandBuffers() { return _commandBuffers; }
			DescriptorLayoutCache& getDescriptorLayoutCache() { return *_descriptorLayoutCache; }
			DescriptorAllocator& getCurrentDescriptorAllocator() { return *_descriptorAllocators[_currentFrame]; }

			/** @return the corresponding thread's command pool to allocate command buffers from */
			std::shared_ptr<CommandPool>& getCommandPool(const std::thread::id &threadID = std::this_thread::get_id()) {
				if (auto it = _commandPools.find(threadID); it != _commandPools.end())
					return it->second;
				return _commandPools.emplace(threadID, std::make_shared<CommandPool>(threadID)).first->second;
			}


			// Helper functions
			/** Wait for every devices to be ready */
			void waitForDevicesReady();


	private:
			// Vulkan values
			VkInstance _instance = VK_NULL_HANDLE;
			VkSurfaceKHR _surface = VK_NULL_HANDLE;

			// Vulkan class values
			CoreWindow& _window;
			std::unique_ptr<CoreDevice> _device;
			std::unique_ptr<Swapchain> _swapchain;

			// Debug callbacks and layers
			std::vector<const char *> _validationLayers { "VK_LAYER_KHRONOS_validation" };
			VkDebugUtilsMessengerEXT _debugMessenger = VK_NULL_HANDLE;


			// Render command objects
			/** Command pools (one for each thread) */
			std::map<std::thread::id, std::shared_ptr<CommandPool>> _commandPools;
			/** Swapchain frames associated command buffers */
			std::vector<std::unique_ptr<CommandBuffer>> _commandBuffers;

			// Descriptors
			/** The cache of every game descriptor layouts */
			std::unique_ptr<DescriptorLayoutCache> _descriptorLayoutCache;
			/** List of every frame descriptor allocator */
			std::vector<std::unique_ptr<DescriptorAllocator>> _descriptorAllocators;


			// Render Sync objects
			/** Max frames being processed at the same time */
			int _currentFramesInFlightCount = 3;
			/** Current drawn frame (% MAX_FRAMES_IN_FLIGHT) */
			std::size_t _currentFrame = 0;



			// Helper functions
			/** @return a list of the extensions required by Vulkan */
			static std::vector<const char *> getRequiredExtensions();
			/** Check if every required extension are available and loaded (avoid VK_ERROR_EXTENSION_NOT_PRESENT) */
			static void hasRequiredExtensions();


			// Debug messages support
			/** @return true if every required validation (= debug) layer are available */
			bool checkValidationLayerSupport() const;
			/**
			 * Update the debug messenger layers struct
			 * @param createInfo The struct that needs to be updated
			 */
			static void populateDebugMessengerCreateInfo(VkDebugUtilsMessengerCreateInfoEXT &createInfo);
	};


	// Helper functions
	/** The debug callback by Vulkan validation layers @return VK_FALSE */
	static VKAPI_ATTR VkBool32 VKAPI_CALL debugCallback(
			VkDebugUtilsMessageSeverityFlagBitsEXT messageSeverity,
			[[maybe_unused]] VkDebugUtilsMessageTypeFlagsEXT messageType,
			const VkDebugUtilsMessengerCallbackDataEXT* pCallbackData,
			void* pUserData);

	/** Creates the Vulkan layers Messenger */
	VkResult createDebugUtilsMessengerEXT(
			VkInstance instance,
			const VkDebugUtilsMessengerCreateInfoEXT* pCreateInfo,
			const VkAllocationCallbacks* pAllocator, VkDebugUtilsMessengerEXT* pDebugMessenger);

	/** Destroy the Vulkan layers Messenger */
	void destroyDebugUtilsMessengerEXT(
			VkInstance instance,
			VkDebugUtilsMessengerEXT debugMessenger,
			const VkAllocationCallbacks *pAllocator);
}
