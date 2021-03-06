#include "TransformModule.hpp"
#include "../GameObject.hpp"

namespace wde::scene {
	TransformModule::TransformModule(GameObject &gameObject) : Module(gameObject, "Transform", ICON_FA_GLOBE) {}

	TransformModule::~TransformModule() {
		WDE_PROFILE_FUNCTION();
		_parent = nullptr;
		_childrenIDs.clear();
	}

	void TransformModule::setConfig(const std::string &data) {
		WDE_PROFILE_FUNCTION();

		auto dataJ = json::parse(data);
		position = glm::vec3 {
			dataJ["position"][0].get<float>(),
			dataJ["position"][1].get<float>(),
			dataJ["position"][2].get<float>()
		};
		rotation = glm::vec3 {
			dataJ["rotation"][0].get<float>(),
			dataJ["rotation"][1].get<float>(),
			dataJ["rotation"][2].get<float>()
		};
		scale = glm::vec3 {
			dataJ["scale"][0].get<float>(),
			dataJ["scale"][1].get<float>(),
			dataJ["scale"][2].get<float>()
		};
	}

	void TransformModule::tick() {};

	void TransformModule::drawGUI() {
#ifdef WDE_GUI_ENABLED
		WDE_PROFILE_FUNCTION();

		gui::GUIRenderer::addVec3Button("Position", position);
		gui::GUIRenderer::addVec3Button("Rotation", rotation);
		gui::GUIRenderer::addVec3Button("Scale", scale, 1.0f);
#endif
	}

	json TransformModule::serialize() {
		WDE_PROFILE_FUNCTION();
		json jData;
		if (_parent != nullptr)
			jData["parentID"] = _parent->_gameObject.getID();
		else
			jData["parentID"] = -1;
		jData["position"] = { position.x, position.y, position.z };
		jData["rotation"] = { rotation.x, rotation.y, rotation.z };
		jData["scale"] = { scale.x, scale.y, scale.z };
		return jData;
	}



	void TransformModule::setParent(TransformModule *parent) {
		WDE_PROFILE_FUNCTION();
		// Remove last parent children ID
		if (_parent != nullptr)
			_parent->_childrenIDs.erase(std::remove(_parent->_childrenIDs.begin(), _parent->_childrenIDs.end(), _gameObject.getID()), _parent->_childrenIDs.end());
		// Change parent
		_parent = parent;
		// Add children to new parent
		_parent->_childrenIDs.push_back(static_cast<int>(_gameObject.getID()));
	}

	glm::mat4 TransformModule::getTransform() const {
		const float c3 = glm::cos(rotation.z);
		const float s3 = glm::sin(rotation.z);
		const float c2 = glm::cos(rotation.x);
		const float s2 = glm::sin(rotation.x);
		const float c1 = glm::cos(rotation.y);
		const float s1 = glm::sin(rotation.y);

		auto mat = glm::mat4{
				{
					 scale.x * (c1 * c3 + s1 * s2 * s3),
				     scale.x * (c2 * s3),
	                 scale.x * (c1 * s2 * s3 - c3 * s1),
                     0.0f,
				},
				{
					 scale.y * (c3 * s1 * s2 - c1 * s3),
			         scale.y * (c2 * c3),
	                 scale.y * (c1 * c3 * s2 + s1 * s3),
                     0.0f,
				},
				{
					 scale.z * (c2 * s1),
				     scale.z * (-s2),
	                 scale.z * (c1 * c2),
                     0.0f,
				},
				{position.x, position.y, position.z, 1.0f}
		};
		if (_parent != nullptr && _parent != this)
			mat = _parent->getTransform() * mat;
		return mat;
	}


	bool TransformModule::decomposeTransform(const glm::mat4& transform, glm::vec3& position, glm::vec3& rotation, glm::vec3& scale) {
		using namespace glm;
		using T = float;
		mat4 LocalMatrix (transform);

		// Normalize matrix
		if (epsilonEqual(LocalMatrix[3][3], static_cast<float>(0), epsilon<T>()))
			return false;

		// Isolate perspective
		if (
				epsilonNotEqual(LocalMatrix[0][3], static_cast<T>(0), epsilon<T>()) ||
				epsilonNotEqual(LocalMatrix[1][3], static_cast<T>(0), epsilon<T>()) ||
				epsilonNotEqual(LocalMatrix[2][3], static_cast<T>(0), epsilon<T>()))
		{
			LocalMatrix[0][3] = LocalMatrix[1][3] = LocalMatrix[2][3] = static_cast<T>(0);
			LocalMatrix[3][3] = static_cast<T>(1);
		}

		// Handle translation
		position = vec3(LocalMatrix[3]);
		LocalMatrix[3] = vec4(0, 0, 0, LocalMatrix[3].w);

		vec3 Row[3], Pdum3;

		// Handle rotation and shear
		for (length_t i = 0; i < 3; i++)
			for (length_t j = 0; j < 3; j++)
				Row[i][j] = LocalMatrix[i][j];

		// Compute X scale factor and normalize first row.
		scale.x = length(Row[0]);
		Row[0] = detail::scale(Row[0], static_cast<T>(1));
		scale.y = length(Row[1]);
		Row[1] = detail::scale(Row[1], static_cast<T>(1));
		scale.z = length(Row[2]);
		Row[2] = detail::scale(Row[2], static_cast<T>(1));

		rotation.y = asin(-Row[0][2]);
		if (cos(rotation.y) != 0) {
			rotation.x = atan2(Row[1][2], Row[2][2]);
			rotation.z = atan2(Row[0][1], Row[0][0]);
		}
		else {
			rotation.x = atan2(-Row[2][0], Row[1][1]);
			rotation.z = 0;
		}
		return true;
	}
}

