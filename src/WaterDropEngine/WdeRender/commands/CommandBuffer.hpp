#pragma once

#include "../../../wde.hpp"

namespace wde::render {
	/**
	 * Class that represents a command buffer.
	 */
	class CommandBuffer : public NonCopyable {
		public:
			// Core functions
			/**
			 * Creates a new command buffer.
			 * @param begin If true, the program will start recording the command buffer
			 * @param bufferLevel The level of the command buffer (Primary or Secondary) - Default is primary
			 */
			explicit CommandBuffer(bool begin, VkCommandBufferLevel bufferLevel = VK_COMMAND_BUFFER_LEVEL_PRIMARY);
			~CommandBuffer() override;


			// Command buffer functions
			/**
			 * Starts the recording state for this command buffer.
			 * @param flags The optional buffer usage flags (default : none)
			 */
			void begin(VkCommandBufferUsageFlags flags = 0);

			/** Ends the recording state for this command buffer. */
			void end();

			/**
			 * Submits the current command buffer to the specified queue
			 * @param fence The fence to signal when submitting is done
			 * @param waitSemaphore The semaphore to wait from before submitting
			 * @param signalSemaphore The semaphore to signal when submitting is done
			 */
			void submit(const VkFence &fence = VK_NULL_HANDLE, const VkSemaphore &waitSemaphore = VK_NULL_HANDLE,
			            const VkSemaphore &signalSemaphore = VK_NULL_HANDLE);

			/** Ends a command buffer, submit the infos to the queue, and wait for the queue */
			void submitIdle();

			/** Wait for the queue to be ready to receive new data */
			void waitForQueueIdle();



			// Getters and setters
			operator const VkCommandBuffer &() const { return _commandBuffer; }
			bool isRunning() const { return _running; }


		private:
			/** True if the command buffer is running */
			bool _running = false;
			/** The buffer level */
			VkCommandBufferLevel _bufferLevel;

			/** The corresponding command buffer */
			VkCommandBuffer _commandBuffer = VK_NULL_HANDLE;

			// Helpers
			static VkQueue getQueue();
	};
}
