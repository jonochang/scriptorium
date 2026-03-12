pub fn admin_intake_script() -> &'static str {
    r#"
  <script>
    let cameraStream = null;
    let scanTimer = null;
    let detector = null;
    let lastScan = "";
    let lastScanAt = 0;
    let intakeStep = 0;
    let resetTimer = null;
    function setScannerStatus(message, tone = "") {
      const panel = document.getElementById("scanner-status");
      panel.textContent = message;
      panel.className = `intake-status-copy${tone ? ` is-${tone}` : ""}`;
    }
    function setLookupStatus(message, tone = "") {
      const panel = document.getElementById("intake-lookup-status");
      panel.textContent = message;
      panel.className = `notice-panel${tone ? ` notice-panel--${tone}` : ""}`;
    }
    function setStep(step) {
      intakeStep = step;
      document.querySelectorAll("[data-step]").forEach((node) => {
        const current = Number(node.dataset.step || "0");
        node.classList.toggle("is-active", current === step);
        node.classList.toggle("is-done", current < step);
        const badge = node.querySelector(".intake-step-badge");
        if (badge) {
          badge.textContent = current < step ? "✓" : String(current + 1);
        }
      });
      document.querySelectorAll("[data-step-connector]").forEach((node) => {
        const current = Number(node.dataset.stepConnector || "0");
        node.classList.toggle("is-done", current < step);
      });
      document.getElementById("intake-review").classList.toggle("is-visible", step >= 1);
      document.getElementById("intake-success").classList.toggle("is-visible", step === 2);
      document.getElementById("intake-reset").hidden = step === 0;
      document.getElementById("intake-hint").hidden = step !== 0;
    }
    function setCameraState(active) {
      document.getElementById("camera-overlay").hidden = !active;
      document.getElementById("camera-empty").hidden = active;
      document.getElementById("camera-stop").hidden = !active;
      document.getElementById("camera-start").disabled = active;
      document.getElementById("camera-start").textContent = active ? "Scanning..." : "Start scanner";
    }
    function setCoverPreview(url, hasStoredAsset = false) {
      const preview = document.getElementById("cover-preview");
      const frame = document.getElementById("cover-frame");
      const placeholder = document.getElementById("cover-placeholder");
      const loaded = document.getElementById("cover-loaded");
      if (url) {
        preview.src = url;
        preview.hidden = false;
        frame.classList.add("has-image");
        placeholder.hidden = true;
        loaded.hidden = !hasStoredAsset;
      } else {
        preview.removeAttribute("src");
        preview.hidden = true;
        frame.classList.remove("has-image");
        placeholder.hidden = false;
        loaded.hidden = true;
      }
    }
    function resetIntakeForm() {
      if (resetTimer) {
        clearTimeout(resetTimer);
        resetTimer = null;
      }
      document.getElementById("isbn").value = "";
      document.getElementById("title").value = "";
      document.getElementById("author").value = "";
      document.getElementById("publisher").value = "";
      document.getElementById("description").value = "";
      document.getElementById("cost-cents").value = "900";
      document.getElementById("retail-cents").value = "1699";
      document.getElementById("initial-stock").value = "5";
      document.getElementById("reorder-point").value = "3";
      document.getElementById("category").value = "Books";
      document.getElementById("vendor").value = "Church Supplier";
      document.getElementById("cover-image-key").value = "";
      document.getElementById("cover-file").value = "";
      setCoverPreview("");
      setLookupStatus("Lookup and save status will appear here.");
      setScannerStatus("Scan a barcode or type an ISBN to begin.");
      setStep(0);
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
      setCameraState(false);
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
        setCameraState(true);
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
            setStep(Math.max(intakeStep, 0));
            setScannerStatus(`Detected ISBN ${barcode.rawValue}. Review and run lookup when ready.`, "success");
          } catch {
            setScannerStatus("Camera is live, but barcode detection needs a steadier frame or better light.", "warning");
          }
        }, 700);
      } catch {
        setCameraState(false);
        setScannerStatus("Camera permission was denied or unavailable. Enter the ISBN manually instead.", "danger");
      }
    }
    async function lookup() {
      const isbn = document.getElementById("isbn").value.trim();
      const token = document.getElementById("token").value;
      if (!token) {
        setLookupStatus("Admin session missing. Sign in again.", "danger");
        return;
      }
      if (!isbn) {
        setLookupStatus("Enter or scan an ISBN before fetching metadata.", "warning");
        return;
      }
      setScannerStatus("Retrieving metadata from Open Library...", "busy");
      setLookupStatus("Fetching metadata...", "warning");
      const res = await fetch("/api/admin/products/isbn-lookup", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ token, isbn }),
      });
      const json = await res.json().catch(() => ({}));
      if (!res.ok) {
        setLookupStatus(json.message || "Metadata lookup failed.", "danger");
        setScannerStatus("Lookup failed. Check the ISBN and try again.", "warning");
        return;
      }
      document.getElementById("title").value = json.title || "";
      document.getElementById("author").value = json.author || "";
      document.getElementById("publisher").value = json.publisher || "";
      document.getElementById("description").value = json.description || "";
      if (json.cover_image_url && !document.getElementById("cover-image-key").value) {
        setCoverPreview(json.cover_image_url, false);
      }
      setStep(1);
      if (json.title) {
        setLookupStatus("Found metadata and auto-filled the product form.", "success");
        setScannerStatus(`✓ ISBN ${isbn} detected. Review the details below.`, "success");
      } else {
        setLookupStatus("No metadata found for that ISBN. You can still fill the form manually.", "warning");
        setScannerStatus(`ISBN ${isbn} detected. Complete the form manually.`, "success");
      }
    }
    async function uploadCover() {
      const token = document.getElementById("token").value;
      const tenantId = document.getElementById("tenant-id").value.trim();
      const fileInput = document.getElementById("cover-file");
      const file = fileInput?.files?.[0];
      if (!token || !tenantId) {
        setLookupStatus("Admin session missing. Sign in again before uploading.", "danger");
        return;
      }
      if (!file) {
        setLookupStatus("Choose an image file before uploading.", "warning");
        return;
      }
      setCoverPreview(URL.createObjectURL(file), false);
      setLookupStatus("Uploading cover...", "warning");
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
        setLookupStatus(json.message || "Cover upload failed.", "danger");
        return;
      }
      document.getElementById("cover-image-key").value = json.object_key || "";
      if (json.asset_url) {
        setCoverPreview(json.asset_url, true);
      } else {
        setCoverPreview(document.getElementById("cover-preview").src, true);
      }
      setLookupStatus("Cover uploaded and ready to save with the product record.", "success");
    }
    async function saveProduct() {
      const token = document.getElementById("token").value;
      const tenantId = document.getElementById("tenant-id").value.trim();
      if (!tenantId) {
        setLookupStatus("Admin session missing. Sign in again to load the tenant before saving inventory.", "danger");
        return;
      }
      const isbn = document.getElementById("isbn").value.trim();
      const title = document.getElementById("title").value.trim();
      if (!title) {
        setLookupStatus("Enter a title before saving the product.", "warning");
        return;
      }
      const category = document.getElementById("category").value.trim() || "Books";
      const vendor = document.getElementById("vendor").value.trim() || "Church Supplier";
      const initialStock = Number(document.getElementById("initial-stock").value || 0);
      setLookupStatus("Saving product...", "warning");
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
        setLookupStatus(json.message || "Save failed.", "danger");
        return;
      }
      let successMessage = `Saved ${json.title || title} for ${category}.`;
      if (initialStock <= 0) {
        successMessage = `${successMessage} No opening stock was received.`;
      } else {
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
        successMessage = receiveRes.ok
          ? `${successMessage} Received opening stock, now on hand ${receiveJson.on_hand ?? initialStock}.`
          : `Saved ${json.title || title}, but stock receive failed: ${receiveJson.message || "unknown error"}.`;
      }
      setLookupStatus(successMessage, "success");
      document.getElementById("intake-success-copy").textContent = successMessage;
      setStep(2);
      resetTimer = setTimeout(() => {
        resetIntakeForm();
      }, 2500);
    }
    document.getElementById("lookup").addEventListener("click", lookup);
    document.getElementById("upload-cover").addEventListener("click", uploadCover);
    document.getElementById("save-product").addEventListener("click", saveProduct);
    document.getElementById("camera-start").addEventListener("click", bootCamera);
    document.getElementById("camera-stop").addEventListener("click", stopCamera);
    document.getElementById("intake-reset").addEventListener("click", resetIntakeForm);
    document.getElementById("isbn").addEventListener("input", (event) => {
      const value = event.target.value.trim();
      if (!value) {
        setScannerStatus("Scan a barcode or type an ISBN to begin.");
      } else if (value.length >= 10) {
        setScannerStatus(`✓ ISBN ${value} detected — click Fetch to pull metadata.`, "success");
      } else {
        setScannerStatus("Keep typing the ISBN or start the scanner.", "busy");
      }
    });
    document.getElementById("cover-file").addEventListener("change", (event) => {
      const file = event.target.files?.[0];
      if (!file) return;
      setCoverPreview(URL.createObjectURL(file), false);
      setLookupStatus("Cover selected. Upload it to store with the product.", "warning");
    });
    window.addEventListener("beforeunload", stopCamera);
    if (document.getElementById("token").value) {
      document.getElementById("intake-auth-status").textContent = "Signed in. You can fetch metadata and save a product.";
      document.getElementById("intake-auth-status").className = "notice-panel notice-panel--success";
    }
    setStep(0);
    setCameraState(false);
    bootCamera();
  </script>
</body>
</html>"#
}
