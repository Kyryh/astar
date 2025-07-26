import init, { Scene, set_panic_hook } from "./astar.js"

/** @type {Scene | undefined} */
let scene;

let drawing = false;


await init().then(function () {
    set_panic_hook()

    /** @type {JQuery<HTMLCanvasElement>} */
    const canvas = $("#canvas")
    const ctx = canvas[0].getContext("2d")
    ctx.imageSmoothingEnabled = false

    requestAnimationFrame(run_pathfinding)

    const start = {
        x: $("#start_x"),
        y: $("#start_y")
    }

    const end = {
        x: $("#end_x"),
        y: $("#end_y")
    }

    const g_cost_multiplier = $("#g_cost_multiplier")
    const h_cost_multiplier = $("#h_cost_multiplier")

    const remove_walls = $("#remove_walls")

    const fast = $("#fast")

    $("#canvas_w").on("change", function () {
        canvas.attr("width", $(this).val())
        ctx.imageSmoothingEnabled = false
    }).trigger("change")

    $("#canvas_h").on("change", function () {
        canvas.attr("height", $(this).val())
        ctx.imageSmoothingEnabled = false
    }).trigger("change")

    function run() {
        scene = new Scene(
            start.x.val(),
            start.y.val(),
            end.x.val(),
            end.y.val(),
            g_cost_multiplier.val(),
            h_cost_multiplier.val(),
            ctx
        )
        scene.init()
    }

    $("#run-button").on("click", run)
    $(document).on("mousedown", function (e) {
        drawing = true;
    })

    $(document).on("mouseup", function (e) {
        drawing = false;
    })

    canvas.on("mousemove", function (evt) {
        if (drawing) {
            draw(evt)
        }
    })

    canvas.on("touchmove", function (evt) {
        draw(evt.changedTouches[0])
    })

    /**
     * @param {Touch | MouseEvent} evt 
     */
    function draw(evt) {
        if (remove_walls.prop('checked')) {
            ctx.fillStyle = "white"
        } else {
            ctx.fillStyle = "black"
        }
        let position = get_mouse_pos(evt)
        ctx.fillRect(position.x, position.y, 1, 1)
        scene = undefined
        if (fast.prop('checked')) {
            run()
        }
    }

    $("input").on("change", function () {
        if (fast.prop('checked')) {
            run()
        }
    })

    if (fast.prop('checked')) {
        run()
    }

    function run_pathfinding() {
        if (scene !== undefined) {
            scene.update(fast.prop('checked'))
        }
        requestAnimationFrame(run_pathfinding)
    }

    function get_mouse_pos(evt) {
        let rect = canvas[0].getBoundingClientRect()
        let scaleX = canvas.attr("width") / rect.width
        let scaleY = canvas.attr("height") / rect.height
        return {
            x: (evt.clientX - rect.left) * scaleX,
            y: (evt.clientY - rect.top) * scaleY
        }
    }
})
