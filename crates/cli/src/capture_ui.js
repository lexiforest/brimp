function installCapturePage(contract, probes, configuration) {
  const button = document.getElementById('capture');
  const status = document.getElementById('status');
  const rows = document.getElementById('results');
  const download = document.getElementById('download');
  let downloadUrl;
  button.addEventListener('click', async () => {
    button.disabled = true;
    download.hidden = true;
    rows.replaceChildren();
    status.textContent = 'Measuring this browser…';
    try {
      const profile = await captureBrowserProfile(contract, probes, createProfileRegistry, {
        ...configuration,
        onProgress({ path, status: state, reason }) {
          const row = document.createElement('tr');
          for (const text of [path, state, reason ?? '']) {
            const cell = document.createElement('td');
            cell.textContent = text;
            row.append(cell);
          }
          if (state === 'captured') row.className = 'captured';
          rows.append(row);
        },
      });
      const json = JSON.stringify(profile, null, 2) + '\n';
      if (downloadUrl) URL.revokeObjectURL(downloadUrl);
      downloadUrl = URL.createObjectURL(new Blob([json], { type: 'application/json' }));
      download.href = downloadUrl;
      download.hidden = false;
      const captured = Object.values(profile.capture.results).filter(
        (r) => r.status === 'captured',
      ).length;
      const summary = `${captured} values captured; ${profile.capture.defaultsUsed.length} defaulted paths.`;
      status.textContent = `${summary} Download your profile below.`;
      const token = new URL(location.href).searchParams.get('token');
      if (token && /^https?:$/.test(location.protocol)) {
        try {
          const response = await fetch('profile', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'X-Brimp-Capture': token },
            body: json,
          });
          const result = await response.json();
          if (!response.ok) throw new Error(result.error ?? `HTTP ${response.status}`);
          status.textContent = `${summary} Saved to ${result.path}. You can close this page.`;
        } catch (error) {
          status.textContent = `${summary} Could not save locally: ${error.message}. Use Download profile.`;
        }
      }
    } catch (error) {
      status.textContent = `Capture failed: ${error.message}`;
    } finally {
      button.disabled = false;
    }
  });
}
