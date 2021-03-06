#include "ControllerModule.hpp"
#include "../../WaterDropEngine.hpp"

namespace wde::scene {
	ControllerModule::ControllerModule(GameObject &gameObject) : Module(gameObject, "Keyboard Controller", ICON_FA_KEYBOARD) {}

	ControllerModule::ControllerModule(GameObject &gameObject, const std::string& data) : Module(gameObject, "Keyboard Controller", ICON_FA_KEYBOARD) {
		WDE_PROFILE_FUNCTION();
		auto dataJ = json::parse(data);
		_moveSpeed = dataJ["moveSpeed"].get<float>();
		_lookSpeed = dataJ["lookSpeed"].get<float>();
	}

	void ControllerModule::tick()  {
		WDE_PROFILE_FUNCTION();
		// Only active if this game object has a camera, and the camera is selected
		auto cameraMod = WaterDropEngine::get().getInstance().getScene()->getActiveCamera();
		if (cameraMod == nullptr || &_gameObject != cameraMod)
			return;

		auto newTime = std::chrono::steady_clock::now();
		auto deltaTime =
				std::chrono::time_point_cast<std::chrono::milliseconds>(newTime).time_since_epoch()
				- std::chrono::time_point_cast<std::chrono::milliseconds>(_lastTime).time_since_epoch();
		_lastTime = newTime;

		// Move in plane XZ given the keyboard inputs
		input::InputController::moveInPlaneXZ((float) deltaTime.count() / 1000.0f, _gameObject, _moveSpeed, _lookSpeed);
	}

	void ControllerModule::drawGUI() {
#ifdef WDE_GUI_ENABLED
		WDE_PROFILE_FUNCTION();
		// Move speed
		gui::GUIRenderer::addFloatDragger("Move speed", _moveSpeed, 1.3f);
		// Look speed
		gui::GUIRenderer::addFloatDragger("Look speed", _lookSpeed, 1.5f);
#endif
	}

	json ControllerModule::serialize() {
		WDE_PROFILE_FUNCTION();
		json jData;
		jData["moveSpeed"] = _moveSpeed;
		jData["lookSpeed"] = _lookSpeed;
		return jData;
	}
}
