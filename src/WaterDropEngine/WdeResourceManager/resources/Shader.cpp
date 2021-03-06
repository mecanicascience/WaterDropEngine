#include "Shader.hpp"
#include "../../WaterDropEngine.hpp"

namespace wde::resource {
	Shader::Shader(const std::string &path)  : Resource(path, ResourceType::SHADER) {
		WDE_PROFILE_FUNCTION();

		// Create shader module
		std::vector<char> shaderContent = WdeFileUtils::readFile(path + ".spv");
		_shaderModule = render::ShaderUtils::createShaderModule(shaderContent);
		_shaderStageType = render::ShaderUtils::getShaderStage(path);
	}

	Shader::~Shader() {
		WDE_PROFILE_FUNCTION();
		// Destroy shader module
		auto device = WaterDropEngine::get().getRender().getInstance().getDevice().getDevice();
		vkDestroyShaderModule(device, _shaderModule, nullptr);
	}

	void Shader::drawGUI() {
#ifdef WDE_GUI_ENABLED
		WDE_PROFILE_FUNCTION();
		ImGui::Text("Shader data ");
		ImGui::Text("  - URL : %s", _path.c_str());
		ImGui::Text("  - Reference Count : %i", _referenceCount);
#endif
	}
}
