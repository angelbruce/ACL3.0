function LabelElement() {
    this.id=""
    this.name=""
    this.classify=ElementType.label
}

function NoumenonElement() {
    this.id=""
    this.name=""
    this.classify=ElementType.noumenon;

    this.time = ""
    this.space= ""
}

function ElementRelation() {
    this.id=""
    this.name=""
    this.classify=""

    this.elements = {}
}

function ElementContext() {
    this.id=""
    this.name=""
    this.classify=""

    this.elements = []
}

var  ElementType = {
    label:"Label",
    noumenon :"Noumenon"
}