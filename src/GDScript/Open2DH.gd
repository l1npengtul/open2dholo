extends Control


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

onready var colorrect = get_node("ColorRect")
onready var main_ui = get_node("Open2GHMainUINode")
onready var panel = get_node("Open2GHMainUINode/Panel")
onready var vbox = get_node("Open2GHMainUINode/Panel/VBoxContainer")


# Called when the node enters the scene tree for the first time.
func _ready():
	self.get_parent().connect("size_changed", self, "resized")

func resized():
	print("a")

# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
