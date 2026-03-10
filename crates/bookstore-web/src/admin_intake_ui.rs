pub fn admin_intake_script() -> &'static str {
    r#"
  <script>
    let cameraStream = null;
    let scanTimer = null;
    let detector = null;
    let lastScan = "";
    let lastScanAt = 0;
    function setScannerStatus(message, tone = "") {
      const panel = document.getElementById("scanner-status");
      panel.textContent = message;
      panel.className = `notice-panel${tone ? ` notice-panel--${tone}` : ""}`;
    }
    async function ensureDetector() {
      if (!("BarcodeDetector" in window)) return null;
      if (detector) return detector;
      const formats = typeof BarcodeDetector.getSupportedFormats === "function"
        ? await BarcodeDetector.getSupportedFormats().catch(() => [])
        : [];
      const preferredFormats = ["ean_13", "ean_8", "upc_a", "upc_e"];
      const activeFormats = formats.length ? preferredFormats.filter((format) => formats.includes(format)) : preferredFormats;
      detector = new BarcodeDetector({ formats: activeFormats.length ? activeFormats : preferredFormats });
      return detector;
    }
    function stopCamera() {
      if (scanTimer) {
        clearInterval(scanTimer);
        scanTimer = null;
      }
      if (cameraStream) {
        cameraStream.getTracks().forEach((track) => track.stop());
        cameraStream = null;
      }
      document.getElementById("camera").srcObject = null;
      setScannerStatus("Scanner stopped. Manual ISBN entry is still available.");
    }
    async function bootCamera() {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        setScannerStatus("Camera access is not available in this browser. Enter the ISBN manually.", "warning");
        return;
      }
      try {
        cameraStream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: { ideal: "environment" } } });
        const video = document.getElementById("camera");
        video.srcObject = cameraStream;
        await video.play().catch(() => {});
        const activeDetector = await ensureDetector();
        if (!activeDetector) {
          setScannerStatus("Camera started. Barcode detection is unavailable here, so type the ISBN manually.", "warning");
          return;
        }
        setScannerStatus("Scanner live. Hold the ISBN barcode steady in frame.");
        scanTimer = setInterval(async () => {
          try {
            const barcodes = await activeDetector.detect(video);
            const barcode = barcodes.find((entry) => entry?.rawValue);
            if (!barcode?.rawValue) return;
            const now = Date.now();
            if (barcode.rawValue === lastScan && now - lastScanAt < 2000) return;
            lastScan = barcode.rawValue;
            lastScanAt = now;
            document.getElementById("isbn").value = barcode.rawValue;
            setScannerStatus(`Detected ISBN ${barcode.rawValue}. Review and run lookup when ready.`, "success");
          } catch {
            setScannerStatus("Camera is live, but barcode detection needs a steadier frame or better light.", "warning");
          }
        }, 700);
      } catch {
        setScannerStatus("Camera permission was denied or unavailable. Enter the ISBN manually instead.", "danger");
      }
    }
    async function lookup() {
      const isbn = document.getElementById("isbn").value;
      const token = document.getElementById("token").value;
      if (!token) {
        document.getElementById("intake-lookup-status").textContent = "Admin session missing. Sign in again.";
        return;
      }
      const res = await fetch("/api/admin/products/isbn-lookup", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ token, isbn }),
      });
      const json = await res.json();
      document.getElementById("title").value = json.title || "";
      document.getElementById("author").value = json.author || "";
      document.getElementById("description").value = json.description || "";
      if (json.cover_image_url && !document.getElementById("cover-image-key").value) {
        const preview = document.getElementById("cover-preview");
        preview.src = json.cover_image_url;
        preview.style.display = "block";
      }
      document.getElementById("intake-lookup-status").textContent = json.title ? "Found metadata and auto-filled the product form." : "No metadata found for that ISBN.";
    }
    async function uploadCover() {
      const token = document.getElementById("token").value;
      const tenantId = document.getElementById("tenant-id").value.trim();
      const fileInput = document.getElementById("cover-file");
      const file = fileInput?.files?.[0];
      if (!token || !tenantId) {
        document.getElementById("intake-lookup-status").textContent = "Admin session missing. Sign in again before uploading.";
        return;
      }
      if (!file) {
        document.getElementById("intake-lookup-status").textContent = "Choose an image file before uploading.";
        return;
      }
      const formData = new FormData();
      formData.append("token", token);
      formData.append("tenant_id", tenantId);
      formData.append("file", file);
      const res = await fetch("/api/admin/products/cover-upload", {
        method: "POST",
        body: formData,
      });
      const json = await res.json().catch(() => ({}));
      if (!res.ok) {
        document.getElementById("intake-lookup-status").textContent = json.message || "Cover upload failed.";
        return;
      }
      document.getElementById("cover-image-key").value = json.object_key || "";
      if (json.asset_url) {
        const preview = document.getElementById("cover-preview");
        preview.src = json.asset_url;
        preview.style.display = "block";
      }
      document.getElementById("intake-lookup-status").textContent = "Cover uploaded and ready to save with the product record.";
    }
    async function saveProduct() {
      const token = document.getElementById("token").value;
      const tenantId = document.getElementById("tenant-id").value.trim();
      if (!tenantId) {
        document.getElementById("intake-lookup-status").textContent = "Admin session missing. Sign in again to load the tenant before saving inventory.";
        return;
      }
      const isbn = document.getElementById("isbn").value.trim();
      const title = document.getElementById("title").value.trim();
      const category = document.getElementById("category").value.trim() || "Books";
      const vendor = document.getElementById("vendor").value.trim() || "Church Supplier";
      const initialStock = Number(document.getElementById("initial-stock").value || 0);
      const res = await fetch("/api/admin/products", {
        method: "POST",
        headers: { "content-type": "application/json", Origin: window.location.origin },
        body: JSON.stringify({
          token,
          tenant_id: tenantId,
          product_id: `prd-${isbn || Date.now()}`,
          title,
          isbn,
          category,
          vendor,
          cost_cents: Number(document.getElementById("cost-cents").value || 0),
          retail_cents: Number(document.getElementById("retail-cents").value || 0),
          cover_image_key: document.getElementById("cover-image-key").value || null,
        }),
      });
      const json = await res.json().catch(() => ({}));
      if (!res.ok) {
        document.getElementById("intake-lookup-status").textContent = json.message || "Save failed.";
        return;
      }
      if (initialStock <= 0) {
        document.getElementById("intake-lookup-status").textContent = `Saved ${json.title || title} for ${category}. No opening stock was received.`;
        return;
      }
      const receiveRes = await fetch("/api/admin/inventory/receive", {
        method: "POST",
        headers: { "content-type": "application/json", Origin: window.location.origin },
        body: JSON.stringify({
          token,
          tenant_id: tenantId,
          isbn,
          quantity: initialStock,
        }),
      });
      const receiveJson = await receiveRes.json().catch(() => ({}));
      document.getElementById("intake-lookup-status").textContent = receiveRes.ok
        ? `Saved ${json.title || title} for ${category}. Received opening stock, now on hand ${receiveJson.on_hand ?? initialStock}.`
        : `Saved ${json.title || title}, but stock receive failed: ${receiveJson.message || "unknown error"}.`;
    }
    document.getElementById("lookup").addEventListener("click", lookup);
    document.getElementById("upload-cover").addEventListener("click", uploadCover);
    document.getElementById("save-product").addEventListener("click", saveProduct);
    document.getElementById("camera-start").addEventListener("click", bootCamera);
    document.getElementById("camera-stop").addEventListener("click", stopCamera);
    window.addEventListener("beforeunload", stopCamera);
    if (document.getElementById("token").value) {
      document.getElementById("intake-auth-status").textContent = "Signed in. You can fetch metadata and save a product.";
      document.getElementById("intake-auth-status").className = "notice-panel notice-panel--success";
    }
    bootCamera();
  </script>
</body>
</html>"#
}
