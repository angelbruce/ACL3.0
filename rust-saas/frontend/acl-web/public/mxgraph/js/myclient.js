// Extends EditorUi to update I/O action states based on availability of backend
(function () {
    var editorUiInit = EditorUi.prototype.init;

    EditorUi.prototype.init = function () {
        editorUiInit.apply(this, arguments);
        this.actions.get('export').setEnabled(false);

        // Updates action states which require a backend
        //if (!Editor.useLocalStorage) {
        //    mxUtils.post(OPEN_URL, '', mxUtils.bind(this, function (req) {
        //        //                var enabled = req.getStatus() != 404;
        //        //                this.actions.get('open').setEnabled(enabled || Graph.fileSupport);
        //        //                this.actions.get('import').setEnabled(enabled || Graph.fileSupport);
        //        //                this.actions.get('save').setEnabled(enabled);
        //        //                this.actions.get('saveAs').setEnabled(enabled);
        //        //                this.actions.get('export').setEnabled(enabled);
        //    }));
        //}

        this.createPopupMenuExtend();
        this.createToolbarExtend();
        this.bindEvents();

    };

    Graph.prototype.createMenu = function () {
    }


    Graph.prototype.getLabel = function (cell) {
        var value = cell.value;
        if (!value) return "";
        if (value.name) return value.name;
        return "";
    }

    Graph.prototype.getLinkForCell = function (cell) {
        return null;
    }


    EditorUi.prototype.bindEvents = function () {

       
    }

    EditorUi.prototype.createPopupMenuExtend = function () {
        this.editor.graph.popupMenuHandler.factoryMethod = mxUtils.bind(this, function (menu, cell, evt) {
            this.menus.createPopupMenu(menu, cell, evt);

            menu.addSeparator();

            var label = menu.addItem('标签', null, mxUtils.bind(this, function (evt) {
                var pos = this.getPointToClient(evt)
                var entity = new LabelElement();
                entity.name = '标签';
                this.createEntity(entity, EditorUi.STYLE_LABEL, pos.x, pos.y, 100, 40, true);
            }));

            var entity = menu.addItem('实体', null, mxUtils.bind(this, function (evt) {
                var pos = this.getPointToClient(evt)
                var entity = new NoumenonElement();
                entity.name = '实体';
                this.createEntity(entity, EditorUi.STYLE_ENTITY, pos.x, pos.y, 80, 25, true);
            }));

            var relation = menu.addItem('关系', null, mxUtils.bind(this, function (evt) {
                var pos = this.getPointToClient(evt)
                var entity = new ElementRelation();
                entity.name = '关系';
                this.createEntity(entity, EditorUi.STYLE_RELATION, pos.x, pos.y, 100, 100, false);
            }));

            var context = menu.addItem('上下文', null, mxUtils.bind(this, function (evt) {
                var pos = this.getPointToClient(evt)
                var entity = new ElementContext();
                entity.name = '上下文';
                this.createEntity(entity, EditorUi.STYLE_CONEXT, pos.x, pos.y, 100, 100, true);
            }));

        });
    }



    EditorUi.prototype.getPointToClient = function (evt) {
        var graph = this.editor.graph;
        var pos = mxUtils.convertPoint(graph.container, evt.clientX, evt.clientY)
        var tr = graph.view.translate;
        var offset = mxUtils.getOffset(graph.container);

        console.log(pos, tr, offset)

        var x = pos.x - tr.x// + offset.x;
        var y = pos.y - tr.y// + offset.y;
        return new mxPoint(x, y);
    }



    EditorUi.prototype.createToolbarExtend = function () {
        var toolbar = this.toolbar
        toolbar.addButton('geIcon geSprite geSprite-plus', '标签', mxUtils.bind(this, function (sender, evt) {
            var entity = new LabelElement();
            entity.name = '标签';
            this.createEntity(entity, EditorUi.STYLE_LABEL, 0, 0, 100, 40, true);
        }));
        toolbar.addButton('geIcon geSprite geSprite-plus', '实体', mxUtils.bind(this, function (sender, evt) {
            var entity = new NoumenonElement();
            entity.name = '实体';
            this.createEntity(entity, EditorUi.STYLE_ENTITY, 0, 0, 80, 25, true);
        }));
        toolbar.addButton('geIcon geSprite geSprite-plus', '关系', mxUtils.bind(this, function (sender, evt) {
            var entity = new ElementRelation();
            entity.name = '关系';
            this.createEntity(entity, EditorUi.STYLE_RELATION, 0, 0, 100, 100, false);
        }));
        toolbar.addButton('geIcon geSprite geSprite-plus', '上下文', mxUtils.bind(this, function (sender, evt) {
            var entity = new ElementContext();
            entity.name = '上下文';
            this.createEntity(entity, EditorUi.STYLE_CONEXT, 0, 0, 100, 100, true);

        }));
    }


    EditorUi.STYLE_LABEL = "shape=swimlane;fontStyle=1;align=center;verticalAlign=top;childLayout=stackLayout;horizontal=1;startSize=26;horizontalStack=0;resizeParent=1;resizeParentMax=0;resizeLast=0;collapsible=1;marginBottom=0;rounded=0;shadow=0;glass=0;comic=0;strokeColor=#333300;fillColor=#FFFFFF;gradientColor=#FFFF33;";
    EditorUi.STYLE_ENTITY = "shape=rectangle;rounded=1;whiteSpace=wrap;html=1;gradientColor=#009999;strokeColor=#003333;";
    EditorUi.STYLE_CONEXT = "shape=swimlane;html=1;horizontal=1;startSize=33;fillColor=#FFFFFF;gradientColor=#007FFF;rounded=0;gradientDirection=west;shadow=0;glass=0;comic=0;swimlaneLine=1;direction=west;verticalAlign=middle;labelPosition=center;verticalLabelPosition=middle;align=center;textDirection=ltr;flipH=1;flipV=0;whiteSpace=wrap;labelBorderColor=none;fontStyle=0";
    EditorUi.STYLE_RELATION = "shape=rounded=0;orthogonalLoop=1;jettySize=auto;html=1;strokeColor=#000099;elbow=vertical;edgeStyle=orthogonalEdgeStyle;curved=1;dashed=1;dashPattern=1 1;shadow=0;comic=0;";

    EditorUi.prototype.createEntity = function (entity, style, x, y, w, h, vertex) {
        var geo = new mxGeometry(x, y, w, h);
        var cell = new mxCell(entity, geo, style)
        cell.setId(null);
        cell.setVertex(vertex ? true : false);
        cell.setEdge(vertex ? false : true);
        cell.setConnectable(true);
        var parent = this.editor.graph.getDefaultParent();
        this.editor.graph.addCell(cell, parent);
    }

    // Adds required resources (disables loading of fallback properties, this can only
    // be used if we know that all keys are defined in the language specific file)
    mxResources.loadDefaultBundle = false;
    var bundle = mxResources.getDefaultBundle(RESOURCE_BASE, mxLanguage) ||
        mxResources.getSpecialBundle(RESOURCE_BASE, mxLanguage);

    console.log([bundle, STYLE_PATH + '/default.xml']);
    // Fixes possible asynchronous requests
    mxUtils.getAll([bundle, STYLE_PATH + '/default.xml'], function (xhr) {
        console.log(xhr[0].getText());
        // Adds bundle text to resources
        mxResources.parse(xhr[0].getText());

        // Configures the default graph theme
        var themes = new Object();
        themes[Graph.prototype.defaultThemeName] = xhr[1].getDocumentElement();

        // Main
        new EditorUi(new Editor(urlParams['chrome'] == '0', themes));
    }, function () {
            document.body.innerHTML = '<center style="margin-top:10%;">Error loading resource files. Please check browser console.</center>';
        });
})();