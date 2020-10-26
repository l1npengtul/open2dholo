extends Tree

onready var scn_edit = get_node("Open2GHMainUINode/Panel/VBoxContainer/HSplitContainer/TabContainer/Scene/Grid/VBoxContainer/Tree")

func _ready():
	var tree_root = scn_edit.create_item()
	tree_root.set_text(0, "ROOT")
	scn_edit.hide_root = true

	tree_root.columns = 2

	var section_model = scn_edit.create_item(tree_root)
	section_model.set_text(0, "Model Properties")
	var item_position_x = scn_edit.create_item(section_model)
	item_position_x.set_text(0, "Model X Position")



