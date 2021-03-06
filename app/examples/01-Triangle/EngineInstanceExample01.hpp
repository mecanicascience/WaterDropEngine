#include "../../../src/WaterDropEngine/WdeCore/Core/WdeInstance.hpp"
#include "PipelineExample01.hpp"

using namespace wde;
using namespace wde::render;

namespace examples {
	class EngineInstanceExample01 : public WdeInstance {
		public:
			void initialize() override {
				setRenderPipeline(std::make_shared<PipelineExample01>());
			}

			void update() override { }

			void cleanUp() override { }
	};
}
