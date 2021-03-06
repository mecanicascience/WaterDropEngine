#include "../../../src/WaterDropEngine/WdeCore/Core/WdeInstance.hpp"
#include "PipelineExample03.hpp"

using namespace wde;
using namespace wde::render;

namespace examples {
	class EngineInstanceExample03 : public WdeInstance {
		public:
			void initialize() override {
				setRenderPipeline(std::make_shared<PipelineExample03>());
			}

			void update() override { }

			void cleanUp() override { }
	};
}
