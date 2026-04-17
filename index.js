import init, { WasmNes } from "./pkg/lakenes_wasm.js";

let nes = null;
let animationFrameId = null;
let audioContext = null;
let wasmMemory = null;

let lastTime = performance.now();
let lastTotalFrames = 0n;
let lastFrameTime = performance.now();
const TARGET_FPS = 60.098; // NES NTSC
const MS_PER_FRAME = 1000 / TARGET_FPS;

const canvas = document.getElementById("game-canvas");
const ctx = canvas.getContext("2d", { alpha: false });
const imageData = ctx.createImageData(256, 240);
const data32 = new Uint32Array(imageData.data.buffer);
const fpsText = document.getElementById("fps-counter");
const inputOverlay = document.getElementById("input-display");

// --- Input Mapping ---
const defaultKeyMap = {
    P1_UP: ["ArrowUp", "w"],
    P1_DOWN: ["ArrowDown", "s"],
    P1_LEFT: ["ArrowLeft", "a"],
    P1_RIGHT: ["ArrowRight", "d"],
    P1_A: ["k", "z"],
    P1_B: ["j", "x"],
    P1_START: ["Enter", "p"],
    P1_SELECT: ["Shift", "l"]
};

const buttonMasks = {
    P1_UP: 0x10, P1_DOWN: 0x20, P1_LEFT: 0x40, P1_RIGHT: 0x80,
    P1_A: 0x1, P1_B: 0x2, P1_START: 0x8, P1_SELECT: 0x4
};

const buttonNames = {
    0x10: "UP", 0x20: "DOWN", 0x40: "LEFT", 0x80: "RIGHT",
    0x1: "A", 0x2: "B", 0x8: "START", 0x4: "SELECT"
};

let userKeyMap = JSON.parse(JSON.stringify(defaultKeyMap));
let activeMappingKey = null;
let currentButtons = 0;

async function run() {
    const wasm = await init();
    wasmMemory = wasm.memory;
    loadSettings();
    setupEventListeners();
    renderPadConfig();
    loadTestRoms();

    // START ASYNC DEBUG LOOP
    setInterval(debugLoop, 100);
}

async function loadTestRoms() {
    try {
        const res = await fetch("https://raw.githubusercontent.com/sammwyy/lakenes/main/roms/testroms.json");
        const list = await res.json();
        const container = document.getElementById("test-rom-list");
        if (!container) return;

        container.innerHTML = `<span class="info-label" style="margin-bottom: 5px;">DEBUG ROMS</span>`;

        list.forEach(path => {
            // path format: "roms/apu/apu_dmc.nes"
            const parts = path.split("/");
            const type = parts[1]; // apu, cpu, ppu
            const filename = parts[2].replace(".nes", "");
            const name = filename.replace(/_/g, " ");

            const div = document.createElement("div");
            div.className = "test-rom-item";
            div.innerHTML = `<span class="test-rom-tag ${type}">[${type.toUpperCase()}]</span> <span>${name}</span>`;

            div.onclick = async () => {
                try {
                    const romRes = await fetch(`https://raw.githubusercontent.com/sammwyy/lakenes/main/${path}`);
                    const blob = await romRes.blob();
                    blob.name = filename + ".nes";
                    loadROM(blob);
                } catch (e) {
                    console.error("Failed to download rom", e);
                }
            };

            container.appendChild(div);
        });
    } catch (e) {
        console.error("Failed to load test roms", e);
    }
}

function renderPadConfig() {
    const container = document.getElementById("pad-config");
    if (!container) return;
    container.innerHTML = "";

    for (const [action, keys] of Object.entries(userKeyMap)) {
        const row = document.createElement("div");
        row.className = "config-row";

        const label = document.createElement("span");
        label.innerText = action.replace("P1_", "");

        const keyGroup = document.createElement("div");
        keyGroup.style.display = "flex";
        keyGroup.style.gap = "5px";

        keys.forEach((key, idx) => {
            const input = document.createElement("div");
            input.className = "key-input";
            input.innerText = key || "NONE";
            input.onclick = (e) => {
                activeMappingKey = { action, idx };
                document.querySelectorAll(".key-input").forEach(el => el.style.borderColor = "var(--glass-border)");
                e.target.innerText = "WAITING...";
                e.target.style.borderColor = "var(--accent-color)";
            };
            keyGroup.appendChild(input);
        });

        row.appendChild(label);
        row.appendChild(keyGroup);
        container.appendChild(row);
    }
}

function setupEventListeners() {
    document.getElementById("rom-input").onchange = (e) => {
        const file = e.target.files[0];
        if (file) loadROM(file);
    };

    window.ondragover = (e) => {
        e.preventDefault();
        const overlay = document.getElementById("drop-overlay");
        if (overlay) overlay.style.display = "flex";
    };
    window.ondragleave = () => {
        const overlay = document.getElementById("drop-overlay");
        if (overlay) overlay.style.display = "none";
    };
    window.ondrop = (e) => {
        e.preventDefault();
        const overlay = document.getElementById("drop-overlay");
        if (overlay) overlay.style.display = "none";
        if (e.dataTransfer.files[0]) loadROM(e.dataTransfer.files[0]);
    };

    window.onkeydown = (e) => {
        if (activeMappingKey) {
            e.preventDefault();
            userKeyMap[activeMappingKey.action][activeMappingKey.idx] = e.key;
            activeMappingKey = null;
            renderPadConfig();
            saveSettings();
            return;
        }
        for (const [action, keys] of Object.entries(userKeyMap)) {
            if (keys.includes(e.key)) currentButtons |= buttonMasks[action];
        }
        updateInputDisplay();
    };

    window.onkeyup = (e) => {
        for (const [action, keys] of Object.entries(userKeyMap)) {
            if (keys.includes(e.key)) currentButtons &= ~buttonMasks[action];
        }
        updateInputDisplay();
    };

    const inputs = [
        "vol-master", "vol-p1", "vol-p2", "vol-tri", "vol-noise", "vol-dmc",
        "speed", "ppu-show-bg", "ppu-show-sp", "ppu-grayscale"
    ];

    inputs.forEach(id => {
        const el = document.getElementById(id);
        if (!el) return;
        el.oninput = el.onchange = () => {
            if (el.type === "range") {
                const labelId = id.replace("vol-", "val-").replace("speed", "val-speed");
                const label = document.getElementById(labelId);
                if (label) label.innerText = `${el.value}%`;
            }
            saveSettings();
            applySettings();
        };
    });
}

function updateInputDisplay() {
    if (!inputOverlay) return;
    const active = [];
    for (const [mask, name] of Object.entries(buttonNames)) {
        if (currentButtons & parseInt(mask)) active.push(name);
    }
    inputOverlay.innerText = `PAD: [ ${active.join(" ") || "NONE"} ]`;
}

function applySettings() {
    if (!nes) return;
    const v = (id) => (document.getElementById(`vol-${id}`)?.value || 100) / 100 * 100;
    nes.set_apu_volumes(v("master"), v("p1"), v("p2"), v("tri"), v("noise"), v("dmc"));
    nes.set_speed(parseInt(document.getElementById("speed")?.value || 100));

    let mask = 0;
    if (document.getElementById("ppu-grayscale")?.checked) mask |= 0x01;
    if (document.getElementById("ppu-show-bg")?.checked) mask |= 0x08;
    if (document.getElementById("ppu-show-sp")?.checked) mask |= 0x10;
    nes.set_ppu_mask_override(mask);
}

function saveSettings() {
    const settings = { controls: userKeyMap };
    const inputs = ["vol-master", "vol-p1", "vol-p2", "vol-tri", "vol-noise", "vol-dmc", "speed", "ppu-show-bg", "ppu-show-sp", "ppu-grayscale"];
    inputs.forEach(id => {
        const el = document.getElementById(id);
        if (el) settings[id] = el.type === "checkbox" ? el.checked : el.value;
    });
    localStorage.setItem("lakenes-settings-v2", JSON.stringify(settings));
}

function loadSettings() {
    const settings = JSON.parse(localStorage.getItem("lakenes-settings-v2") || "{}");
    if (settings.controls) userKeyMap = settings.controls;
    for (const [id, val] of Object.entries(settings)) {
        const el = document.getElementById(id);
        if (el) {
            if (el.type === "checkbox") el.checked = val;
            else el.value = val;
            const label = document.getElementById(id.replace("vol-", "val-").replace("speed", "val-speed"));
            if (label) label.innerText = `${val}%`;
        }
    }
    renderPadConfig();
}

async function loadROM(file) {
    if (animationFrameId) cancelAnimationFrame(animationFrameId);
    const buffer = await file.arrayBuffer();
    nes = WasmNes.new(new Uint8Array(buffer));
    document.getElementById("rom-name").innerText = file.name;
    document.getElementById("info-mapper").innerText = `Mapper ${nes.get_rom_mapper_id()}`;
    if (!audioContext) {
        audioContext = new (window.AudioContext || window.webkitAudioContext)({ latencyHint: 'interactive' });
        setupAudio();
    }
    nes.set_audio_sample_rate(audioContext.sampleRate);
    applySettings();
    lastTime = performance.now();
    lastTotalFrames = 0n;
    lastFrameTime = performance.now();
    loop();
}

function setupAudio() {
    const scriptNode = audioContext.createScriptProcessor(512, 0, 1);
    scriptNode.onaudioprocess = (e) => {
        const output = e.outputBuffer.getChannelData(0);
        if (!nes || nes.is_paused() || nes.audio_buffer_len() < output.length) {
            output.fill(0);
            return;
        }
        for (let i = 0; i < output.length; i++) output[i] = nes.get_audio_sample();
    };
    scriptNode.connect(audioContext.destination);
    window.addEventListener('mousedown', () => audioContext.resume(), { once: true });
}

// --- ASYNC DEBUG LOOP (Runs at 10Hz) ---
const apuHistory = { p1: [], p2: [], tri: [], noise: [], dmc: [] };

function debugLoop() {
    if (!nes || document.getElementById('sidebar').classList.contains('collapsed')) return;

    // CPU Registry Stats (Basic)
    const cyclesEl = document.getElementById("info-cycles");
    const framesEl = document.getElementById("info-frames");
    if (cyclesEl) cyclesEl.innerText = nes.get_total_cycles().toLocaleString();
    if (framesEl) framesEl.innerText = nes.get_total_frames().toLocaleString();

    // Tab Specific Updates
    const activeTab = document.querySelector('.tab-content.active')?.id;
    if (!activeTab) return;

    if (activeTab === "tab-ppu") {
        drawPatternTable(0, "pt-0");
        drawPatternTable(1, "pt-1");
        drawPalette();
    } else if (activeTab === "tab-cpu") {
        updateCPUDebug();
    }

    updateCPUDebug();
}

function updateAPUCanvas(id, key) {
    const c = document.getElementById(id);
    if (!c) return;
    const ctx = c.getContext("2d");
    const history = apuHistory[key];

    ctx.clearRect(0, 0, 100, 20);

    if (history.length === 0) return;

    const currentVal = history[history.length - 1];

    // 1. Actividad Actual en Vivo (Barra Horizontal)
    ctx.fillStyle = "rgba(129, 140, 248, 0.2)";
    ctx.fillRect(0, 0, currentVal * 100, 20);

    // 2. Dibujar el área del historial atrás
    ctx.fillStyle = "rgba(129, 140, 248, 0.4)";
    ctx.beginPath();
    ctx.moveTo(0, 20);
    for (let i = 0; i < history.length; i++) {
        const x = i;
        const y = 20 - (history[i] * 18) - 1;
        ctx.lineTo(x, y);
    }
    ctx.lineTo(history.length - 1, 20);
    ctx.closePath();
    ctx.fill();

    // 3. Dibujar la línea en blanco del historial
    ctx.strokeStyle = "#ffffff";
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let i = 0; i < history.length; i++) {
        const x = i;
        const y = 20 - (history[i] * 18) - 1;
        if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    ctx.stroke();

    // 4. Punto resaltado al final (vivo)
    const lastX = history.length - 1;
    const lastY = 20 - (currentVal * 18) - 1;
    ctx.fillStyle = "#ffffff";
    ctx.beginPath();
    ctx.arc(lastX, lastY, 2, 0, Math.PI * 2);
    ctx.fill();
}

function updateCPUDebug() {
    const [pc, a, x, y, sp, p] = nes.get_cpu_registers();
    document.getElementById("reg-pc").innerText = pc.toString(16).toUpperCase().padStart(4, "0");
    document.getElementById("reg-a").innerText = a.toString(16).toUpperCase().padStart(2, "0");
    document.getElementById("reg-x").innerText = x.toString(16).toUpperCase().padStart(2, "0");
    document.getElementById("reg-y").innerText = y.toString(16).toUpperCase().padStart(2, "0");

    const disasmDiv = document.getElementById("disasm");
    if (disasmDiv) {
        disasmDiv.innerHTML = "";
        let currentAddr = pc;
        for (let i = 0; i < 15; i++) {
            const line = document.createElement("div");
            line.className = "disasm-line" + (i === 0 ? " active" : "");
            const data = nes.disassemble(currentAddr).split("|");
            line.innerHTML = `<span class="disasm-addr">${currentAddr.toString(16).toUpperCase().padStart(4, "0")}</span> ${data[1]}`;
            disasmDiv.appendChild(line);
            currentAddr = parseInt(data[0], 16);
        }
    }
}

function drawPatternTable(idx, canvasId) {
    const pts = nes.get_pattern_table(idx);
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    const imgData = ctx.createImageData(128, 128);
    const palette = nes.get_ppu_palette();
    for (let i = 0; i < pts.length; i++) {
        const color = palette[pts[i]];
        const offset = i << 2;
        imgData.data[offset] = (color >> 16) & 0xFF;
        imgData.data[offset + 1] = (color >> 8) & 0xFF;
        imgData.data[offset + 2] = color & 0xFF;
        imgData.data[offset + 3] = 0xFF;
    }
    ctx.putImageData(imgData, 0, 0);
}

function drawPalette() {
    const palette = nes.get_ppu_palette();
    const container = document.getElementById("palette-view");
    if (!container) return;
    container.innerHTML = "";
    palette.forEach(c => {
        const div = document.createElement("div");
        div.className = "palette-cell";
        div.style.backgroundColor = `rgb(${(c >> 16) & 0xFF},${(c >> 8) & 0xFF},${c & 0xFF})`;
        container.appendChild(div);
    });
}

// --- MAIN EMULATION LOOP ---
function loop() {
    if (!nes) return;

    const now = performance.now();
    const speed = parseInt(document.getElementById("speed")?.value || 100) / 100;
    const adjustedMsPerFrame = MS_PER_FRAME / speed;
    let frameProcessed = false;

    // Process all pending frames
    while (now - lastFrameTime >= adjustedMsPerFrame && !nes.is_paused()) {
        nes.update_joypad(1, currentButtons);

        const startStep = performance.now();
        nes.step_frame();
        const coreTime = performance.now() - startStep;

        // Optimized buffer copy (only for the last frame in the catch-up loop to save time)
        const ptr = nes.get_frame_buffer_ptr();
        const fb = new Uint32Array(wasmMemory.buffer, ptr, 256 * 240);
        for (let i = 0; i < fb.length; i++) {
            const pixel = fb[i];
            data32[i] = 0xFF000000 | ((pixel & 0xFF) << 16) | (pixel & 0xFF00) | ((pixel >> 16) & 0xFF);
        }

        // Live APU Visualizer updates at 60 FPS
        const apuTab = document.getElementById("tab-apu");
        if (apuTab && apuTab.classList.contains("active")) {
            const states = nes.get_apu_channels_state(); // [P1, P2, Tri, Noise, DMC, Master]
            apuHistory.p1.push(states[0]);
            apuHistory.p2.push(states[1]);
            apuHistory.tri.push(states[2]);
            apuHistory.noise.push(states[3]);
            apuHistory.dmc.push(states[4]);

            Object.values(apuHistory).forEach(h => {
                if (h.length > 100) h.shift();
            });

            updateAPUCanvas("view-p1", "p1");
            updateAPUCanvas("view-p2", "p2");
            updateAPUCanvas("view-tri", "tri");
            updateAPUCanvas("view-noise", "noise");
            updateAPUCanvas("view-dmc", "dmc");
            const vu = document.getElementById("vu-master");
            if (vu) vu.style.width = `${Math.min(100, states[5] * 200)}%`;
        }

        if (animationFrameId % 10 === 0) {
            const delayEl = document.getElementById("info-delay");
            if (delayEl) delayEl.innerText = `${coreTime.toFixed(2)}ms core`;
        }

        lastFrameTime += adjustedMsPerFrame;
        frameProcessed = true;
    }

    if (frameProcessed) {
        ctx.putImageData(imageData, 0, 0);
    }

    // Reset loop if it drifts too much (e.g. tab was inactive)
    if (now - lastFrameTime > 100) {
        lastFrameTime = now;
    }

    if (now - lastTime >= 1000) {
        const totalFrames = nes.get_total_frames();
        const diff = Number(totalFrames - lastTotalFrames);
        if (fpsText) fpsText.innerText = `${diff} FPS`;
        lastTotalFrames = totalFrames;
        lastTime = now;
    }

    animationFrameId = requestAnimationFrame(loop);
}

window.togglePause = () => {
    nes?.set_paused(!nes.is_paused());
    const btn = document.getElementById("btn-pause");
    if (btn) btn.innerText = nes.is_paused() ? "RESUME" : "PAUSE";
};

window.toggleSidebar = () => document.getElementById('sidebar').classList.toggle('collapsed');

window.showTab = (id, event) => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

    // Find button to set active style
    const btn = event ? event.currentTarget : document.querySelector(`.tab-btn[onclick*="${id}"]`);
    if (btn) btn.classList.add('active');

    document.getElementById('tab-' + id)?.classList.add('active');
};

run();
