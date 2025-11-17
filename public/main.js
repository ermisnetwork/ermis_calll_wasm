import init, { ErmisCall } from "./wasm/ermis_call_node_wasm.js";

await init();

log("launching iroh endpoint …");

const node = new ErmisCall();
await node.spawn(["https://test-iroh.ermis.network.:8443"]);

log("iroh endpoint launched");
let endpointAddr = await node.getLocalEndpointAddr();
log("our endpoint addr: " + endpointAddr);

log("connect from the command line:");
log("git clone https://github.com/n0-computer/iroh-examples.git", "cmd");
log("cd iroh-examples/browser-echo", "cmd");
log(`cargo run --features cli -- connect ${endpointAddr} "hi from cli"`, "cmd");
const link = createConnectLink(endpointAddr, "hi from browser");
log(`connect from the browser: ${link}`);
log("waiting for connections …");

// show the form and connection logs
document.querySelector(".spawned").style = "display: block";

// initiate outgoing connections on form submit
document.querySelector("form#connect").onsubmit = onConnectSubmit;

// handle sending data
document.querySelector("form#send-data").onsubmit = onSendDataSubmit;

// fill the connect form
fillFormFromUrlAndSubmit();

// Connection state
let isConnected = false;
let currentPeerAddr = null;

// Auto accept incoming connections
(async () => {
  log("listening for incoming connections...");
  try {
    await node.acceptConnection();
    log("incoming connection accepted", "success");
    await node.acceptBidiStream();
    log("bidirectional stream established", "success");
    isConnected = true;

    // Enable send form
    document.querySelector("form#send-data").style.display = "block";

    // Start receiving messages
    receiveMessages();

    // Start stats monitoring
    // startStatsMonitoring();
  } catch (err) {
    log(`failed to accept connection: ${err}`, "error");
  }
})();

let receiveCounter = 0;
let now = performance.now();
// Receive messages continuously
async function receiveMessages() {
  const $incoming = document.querySelector("#incoming");
  while (isConnected) {
    try {
      const data = await node.asyncRecv();
      const text = new TextDecoder().decode(data);
      if (text.startsWith("[ermis-call]")) {
        // skip internal messages
        logNodeEvent($incoming, "peer", `internal message: ${text}`, "info");
      }
      receiveCounter += 1;
      if (performance.now() - now >= 1000) {
        logNodeEvent(
          $incoming,
          "peer",
          `Receiving rate: ${receiveCounter} messages/sec, data size: ${data.length} bytes`,
          "info",
          $incoming
        );
        receiveCounter = 0;
        now = performance.now();
      }
      // logNodeEvent($incoming, "peer", `received: ${text}`, "success");
    } catch (err) {
      log(`receive error: ${err}`, "error");
      break;
    }
  }
}

// Stats monitoring
function startStatsMonitoring() {
  const $stats = document.querySelector("#stats");
  setInterval(() => {
    try {
      const stats = node.getStats();
      const html = `
        <div class="stat-item">Connection: ${
          stats.connection_type || "N/A"
        }</div>
        <div class="stat-item">RTT: ${
          stats.rtt_ms ? stats.rtt_ms.toFixed(2) + " ms" : "N/A"
        }</div>
        <div class="stat-item">Packet Loss: ${
          stats.packet_loss ? (stats.packet_loss * 100).toFixed(2) + "%" : "N/A"
        }</div>
      `;
      $stats.innerHTML = html;
    } catch (err) {
      console.error("stats error", err);
    }
  }, 1000);
}

// Handle outgoing connections
async function onConnectSubmit(e) {
  console.log("connect form submitted", e);
  e.preventDefault();
  const data = new FormData(e.target);
  const peerAddr = data.get("endpoint-id");
  const payload = data.get("payload");
  console.log("connect form data", { peerAddr, payload });
  if (!peerAddr) return;

  const $outgoing = document.querySelector("#outgoing");
  try {
    logNodeEvent($outgoing, peerAddr, "connecting …");
    console.log("connecting to peer", peerAddr);
    // Connect to peer
    await node.connect(peerAddr);
    logNodeEvent($outgoing, peerAddr, "connected", "success");
    currentPeerAddr = peerAddr;

    // Open bidirectional stream
    await node.openBidiStream();
    logNodeEvent($outgoing, peerAddr, "stream opened", "success");
    isConnected = true;

    // Send initial payload if provided
    if (payload) {
      const encoder = new TextEncoder();
      await node.asyncSend(encoder.encode(payload));
      logNodeEvent($outgoing, peerAddr, `sent: ${payload}`, "success");
    }

    // Enable send form
    document.querySelector("form#send-data").style.display = "block";

    // Start receiving messages
    receiveMessages();

    // Start stats monitoring
    // startStatsMonitoring();
  } catch (err) {
    logNodeEvent($outgoing, peerAddr, `connection failed: ${err}`, "error");
  }
}

// Handle sending data through the form
async function onSendDataSubmit(e) {
  e.preventDefault();
  if (!isConnected) {
    log("not connected to any peer", "error");
    return;
  }

  const data = new FormData(e.target);
  const message = data.get("message");
  if (!message) return;

  const $outgoing = document.querySelector("#outgoing");
  try {
    const sendmessage = `[ermis-call]:${message}`;
    const encoder = new TextEncoder();
    await node.asyncSend(encoder.encode(sendmessage));
    logNodeEvent(
      $outgoing,
      currentPeerAddr || "peer",
      `sent: ${message}`,
      "success"
    );

    // Clear the input
    e.target.querySelector("[name=message]").value = "";
  } catch (err) {
    log(`send error: ${err}`, "error");
  }
}

// Send dummy data periodically
let dummyDataInterval = null;
function startDummyData() {
  if (dummyDataInterval) return;

  const $outgoing = document.querySelector("#outgoing");
  let counter = 0;

  let dummyCounter = 0;
  let dummyStartTime = performance.now();
  dummyDataInterval = setInterval(async () => {
    if (!isConnected) {
      stopDummyData();
      return;
    }

    try {
      // const message = `Dummy message #${++counter} at ${new Date().toISOString()}`;
      // const encoder = new TextEncoder();
      // dummy 1000 bytes message
      const message = new Uint8Array(5000);
      await node.asyncSend(message);
      dummyCounter += 1;
      if (performance.now() - dummyStartTime >= 1000) {
        logNodeEvent(
          $outgoing,
          currentPeerAddr || "peer",
          `sent dummy: ${dummyCounter} messages/sec`,
          "info"
        );
        dummyCounter = 0;
        dummyStartTime = performance.now();
      }
    } catch (err) {
      log(`dummy send error: ${err}`, "error");
      stopDummyData();
    }
  }, 10); // Send every 20 milliseconds

  log("started sending dummy data (every 2s)", "info");
}

async function startPublishingVideo() {
  if (!isConnected) {
    log("not connected to any peer", "error");
    return;
  }
  const userMedia = await navigator.mediaDevices.getUserMedia({
    video: true,
    audio: false,
  });
  const videoTrack = userMedia.getVideoTracks()[0];
  const mediaStreamTrackProcessor = new MediaStreamTrackProcessor({
    track: videoTrack,
  });
  const reader = mediaStreamTrackProcessor.readable.getReader();

  const videoEncoder = new VideoEncoder({
    output: async (chunk, metadata) => {
      // Send encoded video chunk
      try {
        const arrayBuffer = new ArrayBuffer(chunk.byteLength);
        chunk.copyTo(new Uint8Array(arrayBuffer));
        await node.asyncSend(new Uint8Array(arrayBuffer));
        log(`sent video chunk, size: ${chunk.byteLength}`, "info");
      } catch (err) {
        log(`video send error: ${err}`, "error");
      }
    },
    error: (error) => {
      log(`VideoEncoder error: ${error}`, "error");
    },
  });

  videoEncoder.configure({
    codec: "avc1.42E01E", // H.264 baseline profile
    width: 1280,
    height: 720,
    bitrate: 1_000_000,
    framerate: 30,
  });
  
  const videoElement = document.querySelector("#local-video");
  videoElement.srcObject = new MediaStream([videoTrack]);
  videoElement.play();
  let frameCounter = 0;
  while (isConnected) {
    try {
      const result = await reader.read();
      if (result.done) break;
      const videoFrame = result.value;
      frameCounter += 1;
      let keyframe = frameCounter % 30 === 0; // Force keyframe every 150 frames
      videoEncoder.encode(videoFrame, { keyFrame: keyframe });
      videoFrame.close();

      // Convert VideoFrame to ArrayBuffer (you may want to use a more efficient method)
    } catch (err) {
      log(`video send error: ${err}`, "error");
      break;
    }
  }

  
}

function stopDummyData() {
  if (dummyDataInterval) {
    clearInterval(dummyDataInterval);
    dummyDataInterval = null;
    log("stopped sending dummy data", "info");
  }
}

// Attach dummy data controls
document.querySelector("#start-dummy").onclick = startDummyData;
document.querySelector("#stop-dummy").onclick = stopDummyData;

function log(line, className, parent) {
  const time = new Date().toISOString().substring(11, 22);
  if (!parent) parent = document.querySelector("main");
  const el = document.createElement("div");
  line = `<span class=time>${time}: </span>${line}`;
  el.innerHTML = line;
  if (className) el.classList.add(className);
  parent.appendChild(el);

  // Auto scroll to bottom
  parent.scrollTop = parent.scrollHeight;
}

function logNodeEvent(container, endpointId, event, className) {
  let nodeDiv = container.querySelector(`.node-01`);
  if (!nodeDiv) {
    nodeDiv = document.createElement("div");
    nodeDiv.classList.add("node");
    nodeDiv.classList.add(`node-01`);
    const heading = document.createElement("h3");
    heading.innerText = endpointId;
    nodeDiv.appendChild(heading);
    container.appendChild(nodeDiv);
  }
  log(`${event}`, className, nodeDiv);
}

function fillFormFromUrlAndSubmit() {
  const $form = document.querySelector("form#connect");
  const url = new URL(document.location);
  $form.querySelector("[name=endpoint-id]").value =
    url.searchParams.get("connect") || "";
  $form.querySelector("[name=payload]").value =
    url.searchParams.get("payload") || "";

  // Only auto-submit if we have connect parameter
  if (url.searchParams.get("connect")) {
    document.querySelector("form#connect").requestSubmit();
  }
}

function createConnectLink(endpointAddr, payload) {
  const ourUrl = new URL(document.location);
  ourUrl.searchParams.set("connect", endpointAddr);
  ourUrl.searchParams.set("payload", payload);
  return `<a href="${ourUrl}" target="_blank">click here</a>`;
}
