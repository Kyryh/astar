import init, { Scene, set_panic_hook } from "./astar.js"

/** @type {Scene | undefined} */
let scene;

let drawing = false;


function get_mouse_pos(canvas, evt) {
    let rect = canvas.getBoundingClientRect()
    let scaleX = canvas.width / rect.width
    let scaleY = canvas.height / rect.height
    return {
        x: (evt.clientX - rect.left) * scaleX,
        y: (evt.clientY - rect.top) * scaleY
    }
}


await init().then(function () {
    set_panic_hook()

    /** @type {HTMLCanvasElement} */
    const canvas = document.querySelector("#canvas")
    const ctx = canvas.getContext("2d")
    ctx.imageSmoothingEnabled = false

    requestAnimationFrame(run_pathfinding)

    const start = {
        x: document.querySelector("#start_x"),
        y: document.querySelector("#start_y")
    }

    const end = {
        x: document.querySelector("#end_x"),
        y: document.querySelector("#end_y")
    }

    const g_cost_multiplier = document.querySelector("#g_cost_multiplier")
    const h_cost_multiplier = document.querySelector("#h_cost_multiplier")

    const remove_walls = document.querySelector("#remove_walls")

    const fast = document.querySelector("#fast")


    function run() {
        scene = new Scene(
            start.x.value,
            start.y.value,
            end.x.value,
            end.y.value,
            g_cost_multiplier.value,
            h_cost_multiplier.value,
            ctx
        )
        scene.init()
    }


    document
        .querySelector("#run-button")
        .addEventListener("click", run)
    document
        .addEventListener("mousedown", function (e) {
            drawing = true;
        })

    document
        .addEventListener("mouseup", function (e) {
            drawing = false;
        })

    canvas.addEventListener("mousemove", function (evt) {
        if (drawing) {
            draw(evt)
        }
    })

    canvas.addEventListener("touchmove", function (evt) {
        draw(evt.changedTouches[0])
    })


    /**
     * @param {Touch | MouseEvent} evt 
     */
    function draw(evt) {
        if (remove_walls.checked) {
            ctx.fillStyle = "white"
        } else {
            ctx.fillStyle = "black"
        }
        let position = get_mouse_pos(canvas, evt)
        ctx.fillRect(position.x, position.y, 1, 1)
        scene = undefined
        if (fast.checked) {
            run()
        }
    }

    document.querySelectorAll("input").forEach(function (input) {
        input.addEventListener("change", function () {
            if (fast.checked) {
                run()
            }
        })
    })

    if (fast.checked) {
        run()
    }

    function run_pathfinding() {
        if (scene !== undefined) {
            scene.update(fast.checked)
        }
        requestAnimationFrame(run_pathfinding)
    }

})
