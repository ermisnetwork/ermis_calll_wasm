import init, { ErmisCall } from "./wasm/ermis_call_node_wasm.js";

const RELAYS = ["https://test-iroh.ermis.network.:8443"];
const encoder = new TextEncoder();
const decoder = new TextDecoder();

await init();

const node = new ErmisCall();
let endpointAddr = "";
let isConnected = false;
let currentPeerAddr = "";
let receiveLoopStarted = false;
let audioInterval = null;
let audioSeq = 0;
let videoSeq = 0;
let videoGopStarted = false;
let receivedCount = 0;
let receivedBytes = 0;
let isAccepting = false;

const $endpointAddr = document.querySelector("#endpoint-addr");
const $connectionState = document.querySelector("#connection-state");
const $sendData = document.querySelector("#send-data");
const $outgoing = document.querySelector("#outgoing");
const $incoming = document.querySelector("#incoming");
const $stats = document.querySelector("#stats");

document.querySelector(".spawned").style.display = "block";
document.querySelector("form#connect").onsubmit = onConnectSubmit;
document.querySelector("form#send-data").onsubmit = onSendDataSubmit;
document.querySelector("#copy-addr").onclick = copyEndpointAddr;
document.querySelector("#accept-incoming").onclick = acceptIncomingConnection;
document.querySelector("#send-audio").onclick = sendAudioTestFrame;
document.querySelector("#start-audio").onclick = startAudioTest;
document.querySelector("#stop-audio").onclick = stopAudioTest;
document.querySelector("#send-video-keyframe").onclick = sendVideoKeyframe;
document.querySelector("#send-video-frame").onclick = sendVideoFrame;

setConnected(false);
log("launching iroh endpoint...");

try {
  await node.spawn(RELAYS);
  endpointAddr = await node.getLocalEndpointAddr();
  $endpointAddr.value = endpointAddr;
  log("iroh endpoint launched", "success");
  log("ready. On the receiving tab, click Accept Incoming before connecting from another tab.");
  fillFormFromUrlAndSubmit();
  startStatsMonitoring();
} catch (err) {
  log(`failed to launch endpoint: ${formatError(err)}`, "error");
}

async function acceptIncomingConnection() {
  if (isConnected || isAccepting) return;

  isAccepting = true;
  document.querySelector("#accept-incoming").disabled = true;
  log("waiting for incoming connection...");

  try {
    await node.acceptConnection();
    currentPeerAddr = "incoming peer";
    setConnected(true);
    logNodeEvent($incoming, currentPeerAddr, "connection accepted", "success");
    startReceiveLoop();
  } catch (err) {
    log(`accept connection failed: ${formatError(err)}`, "error");
  } finally {
    isAccepting = false;
    document.querySelector("#accept-incoming").disabled = isConnected;
  }
}

async function onConnectSubmit(event) {
  event.preventDefault();
  const form = new FormData(event.target);
  const peerAddr = form.get("endpoint-id")?.trim();
  const payload = form.get("payload")?.trim();
  if (!peerAddr) return;

  try {
    logNodeEvent($outgoing, peerAddr, "connecting...");
    await node.connect(peerAddr);
    currentPeerAddr = peerAddr;
    setConnected(true);
    logNodeEvent($outgoing, peerAddr, "connected", "success");
    startReceiveLoop();

    if (payload) {
      sendControlPayload(payload);
    }
  } catch (err) {
    logNodeEvent($outgoing, peerAddr, `connect failed: ${formatError(err)}`, "error");
  }
}

function onSendDataSubmit(event) {
  event.preventDefault();
  if (!ensureConnected()) return;

  const message = new FormData(event.target).get("message")?.trim();
  if (!message) return;

  sendControlPayload(message);
  event.target.reset();
}

function sendControlPayload(message) {
  const payload = `[control] ${message}`;
  node.sendControlFrame(encoder.encode(payload));
  logNodeEvent($outgoing, currentPeerAddr || "peer", `sent control: ${message}`, "success");
}

function sendAudioTestFrame() {
  if (!ensureConnected()) return;

  const frame = makeTestFrame("AUDT", audioSeq++);
  node.sendAudioFrame(frame);
  logNodeEvent($outgoing, currentPeerAddr || "peer", `sent audio frame #${audioSeq}`, "success");
}

function startAudioTest() {
  if (!ensureConnected() || audioInterval) return;

  audioInterval = setInterval(() => {
    const frame = makeTestFrame("AUDT", audioSeq++);
    node.sendAudioFrame(frame);
  }, 20);
  logNodeEvent($outgoing, currentPeerAddr || "peer", "started audio stream test at 50 fps", "info");
}

function stopAudioTest() {
  if (!audioInterval) return;

  clearInterval(audioInterval);
  audioInterval = null;
  logNodeEvent($outgoing, currentPeerAddr || "peer", "stopped audio stream test", "info");
}

function sendVideoKeyframe() {
  if (!ensureConnected()) return;

  const frame = makeTestFrame("VIDK", videoSeq++);
  node.beginWithGop(frame);
  videoGopStarted = true;
  logNodeEvent($outgoing, currentPeerAddr || "peer", `sent video keyframe #${videoSeq}`, "success");
}

function sendVideoFrame() {
  if (!ensureConnected()) return;

  if (!videoGopStarted) {
    sendVideoKeyframe();
    return;
  }

  const frame = makeTestFrame("VIDF", videoSeq++);
  node.sendFrame(frame);
  logNodeEvent($outgoing, currentPeerAddr || "peer", `sent video frame #${videoSeq}`, "success");
}

async function startReceiveLoop() {
  if (receiveLoopStarted) return;
  receiveLoopStarted = true;

  while (true) {
    try {
      const data = await node.asyncRecv();
      receivedCount += 1;
      receivedBytes += data.length;
      renderReceivedFrame(data);
    } catch (err) {
      log(`receive error: ${formatError(err)}`, "error");
      receiveLoopStarted = false;
      setConnected(false);
      return;
    }
  }
}

function renderReceivedFrame(data) {
  const kind = decoder.decode(data.slice(0, 4));
  const seq = data.length >= 8 ? new DataView(data.buffer, data.byteOffset + 4, 4).getUint32(0) : 0;

  if (kind === "AUDT") {
    logNodeEvent($incoming, "peer", `received audio frame #${seq}, ${data.length} bytes`, "success");
    return;
  }

  if (kind === "VIDK") {
    logNodeEvent($incoming, "peer", `received video keyframe #${seq}, ${data.length} bytes`, "success");
    return;
  }

  if (kind === "VIDF") {
    logNodeEvent($incoming, "peer", `received video frame #${seq}, ${data.length} bytes`, "success");
    return;
  }

  const text = decoder.decode(data);
  logNodeEvent($incoming, "peer", `received: ${text || `${data.length} raw bytes`}`, "info");
}

function makeTestFrame(kind, seq) {
  const frame = new Uint8Array(160);
  frame.set(encoder.encode(kind), 0);
  new DataView(frame.buffer).setUint32(4, seq);

  for (let i = 8; i < frame.length; i += 1) {
    frame[i] = (seq + i) % 256;
  }

  return frame;
}

function setConnected(value) {
  isConnected = value;
  $connectionState.textContent = value ? "Connected" : "Not connected";
  $connectionState.className = value ? "status connected" : "status";
  $sendData.style.display = value ? "flex" : "none";
  document.querySelector("#media-tests").style.display = value ? "block" : "none";
  document.querySelector("#accept-incoming").disabled = value || isAccepting;
}

function ensureConnected() {
  if (isConnected) return true;
  log("not connected to a peer", "error");
  return false;
}

async function copyEndpointAddr() {
  if (!endpointAddr) return;

  try {
    await navigator.clipboard.writeText(endpointAddr);
    log("endpoint address copied", "success");
  } catch {
    $endpointAddr.select();
    document.execCommand("copy");
    log("endpoint address selected/copied", "success");
  }
}

function startStatsMonitoring() {
  setInterval(() => {
    try {
      const stats = node.getStats();
      $stats.innerHTML = `
        <div class="stat-item">Connection: ${stats.connection_type || "N/A"}</div>
        <div class="stat-item">RTT: ${stats.rtt_ms ? stats.rtt_ms.toFixed(2) + " ms" : "N/A"}</div>
        <div class="stat-item">Packet loss: ${
          Number.isFinite(stats.packet_loss) ? `${(stats.packet_loss * 100).toFixed(2)}%` : "N/A"
        }</div>
        <div class="stat-item">Received: ${receivedCount} frames / ${receivedBytes} bytes</div>
      `;
    } catch {
      $stats.innerHTML = `
        <div class="stat-item">Connection: N/A</div>
        <div class="stat-item">RTT: N/A</div>
        <div class="stat-item">Packet loss: N/A</div>
        <div class="stat-item">Received: ${receivedCount} frames / ${receivedBytes} bytes</div>
      `;
    }
  }, 1000);
}

function fillFormFromUrlAndSubmit() {
  const form = document.querySelector("form#connect");
  const url = new URL(document.location);
  const connectAddr = url.searchParams.get("connect");

  form.querySelector("[name=endpoint-id]").value = connectAddr || "";
  form.querySelector("[name=payload]").value = url.searchParams.get("payload") || "";

  if (connectAddr) {
    form.requestSubmit();
  }
}

function log(line, className, parent) {
  const time = new Date().toISOString().substring(11, 22);
  const el = document.createElement("div");
  el.innerHTML = `<span class="time">${time}: </span>${line}`;
  if (className) el.classList.add(className);
  (parent || document.querySelector("main")).appendChild(el);
  (parent || document.querySelector("main")).scrollTop = (parent || document.querySelector("main")).scrollHeight;
}

function logNodeEvent(container, endpointId, event, className) {
  let nodeDiv = container.querySelector(".node-01");
  if (!nodeDiv) {
    nodeDiv = document.createElement("div");
    nodeDiv.classList.add("node", "node-01");

    const heading = document.createElement("h3");
    heading.innerText = endpointId;
    nodeDiv.appendChild(heading);
    container.appendChild(nodeDiv);
  }

  log(event, className, nodeDiv);
}

function formatError(err) {
  return err?.message || String(err);
}
