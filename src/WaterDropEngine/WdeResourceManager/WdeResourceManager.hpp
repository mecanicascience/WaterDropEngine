#pragma once

#include <utility>
#include <queue>

#include "../../wde.hpp"
#include "../WdeCore/Core/Module.hpp"
#include "Resource.hpp"
#include "../WdeGUI/panels/ResourcesPanel.hpp"

namespace wde::resource {
	/**
	 * Resource manager module
	 */
	class WdeResourceManager : public core::Module {
		public:
			// Core methods
			explicit WdeResourceManager(std::shared_ptr<core::Subject> moduleSubject);
			void tick() override;
			void cleanUp() override;
			void onNotify(const core::Event& event) override;


			// Resources getter
			/**
			 * Load a given resource
			 * @tparam T Type of the resource
			 * @param resource The path to the resource
			 * @return A pointer to the resource
			 */
			template<typename T>
			T* load(const std::string& resource) {
				// Resource already imported
				if (_resources.contains(resource)) {
					auto res = _resources[resource];
					res->increaseReferenceCount();
					return std::dynamic_pointer_cast<T>(res).get();
				}

				// Create resource
				logger::log(LogLevel::DEBUG, LogChannel::RES) << "Loading resource \"" << resource << "\"." << logger::endl;
				std::shared_ptr<T> baseRes (new T(resource));
				auto res = std::dynamic_pointer_cast<Resource>(baseRes);
				res->increaseReferenceCount();
				_resources[resource] = res;
				_resourcesByType[res->getType()][resource] = res;
				return baseRes.get();
			}

			/**
			 * Release a given resource
			 * @param resource The path to the resource
			 */
			void release(const std::string& resource) {
				// Not loaded in memory
				if (!_resources.contains(resource))
					return;

				// Decrease reference count
				auto& res = _resources.at(resource);
				if (res == nullptr)
					return;
				res->decreaseReferenceCount();

				// Release resource if it can (3 ticks remaining)
				if (res->getReferenceCount() <= 0 && !_resourcesToDelete.contains(res->getPath()))
					_resourcesToDelete.emplace(res->getPath(), 3);
			}


			// Getters and setters
			gui::ResourcesPanel& getResourcesPanel() { return _resourcesPanel; }
			std::unordered_map<Resource::ResourceType, std::unordered_map<std::string, std::shared_ptr<Resource>>>& getResourcesByType() { return _resourcesByType; }



		private:
			// Resources
			/** Resources list by path */
			std::unordered_map<std::string, std::shared_ptr<Resource>> _resources {};
			/** Resources list by type */
			std::unordered_map<Resource::ResourceType, std::unordered_map<std::string, std::shared_ptr<Resource>>> _resourcesByType {};
			/** Resources list that needs to be deleted by path (name - tickRemainingBeforeDeleting) */
			std::unordered_map<std::string, int> _resourcesToDelete {};

			// GUI
			/** The resources GUI panel */
			gui::ResourcesPanel _resourcesPanel {};
	};
}
