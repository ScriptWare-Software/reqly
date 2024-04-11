<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { writable } from "svelte/store";
  
    let url = "";
    let method = "GET";
    let headers = "";
    let body = "";
    let responseMsg = "";
  
    const methods = ["GET", "POST", "PUT", "DELETE"];
  
    async function sendRequest() {
      try {
        const headersArray = headers.split("\n").filter(h => h.trim() !== "");
        const request = { url, method, headers: headersArray, body };
        const response = await invoke("perform_http_request", { request });
        responseMsg = JSON.stringify(response, null, 2);
      } catch (error) {
        responseMsg = `Error: ${error}`;
      }
    }
  </script>
  
  <div>
    <h2>HTTP Request Test</h2>
    <form on:submit|preventDefault={sendRequest}>
      <input type="text" bind:value={url} placeholder="URL" />
      <select bind:value={method}>
        {#each methods as m}
          <option value={m}>{m}</option>
        {/each}
      </select>
      <textarea bind:value={headers} placeholder="Headers (one per line)"></textarea>
      <textarea bind:value={body} placeholder="Body"></textarea>
      <button type="submit">Send Request</button>
    </form>
    <pre>{responseMsg}</pre>
  </div>
  
  <style>
    /* Add your styles here */
  </style>
  