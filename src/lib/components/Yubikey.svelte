<!-- YubiKey.svelte - YubiKey management component -->
<script>
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  
  let yubikeys = $state([]);
  let selectedYubikey = $state(null);
  let challenge = $state('');
  let message = $state('');
  let status = $state('');
  let loading = $state(false);
  let textToEncrypt = $state('');
  let encryptedText = $state('');
  
  onMount(async () => {
    try {
      await listYubikeys();
    } catch (error) {
      status = `Error: ${error}`;
    }
  });
  
  async function listYubikeys() {
    loading = true;
    try {
      yubikeys = await invoke('list_yubikeys');
      if (yubikeys.length > 0) {
        selectedYubikey = yubikeys[0];
        status = 'YubiKeys detected!';
      } else {
        status = 'No YubiKeys found. Please insert a YubiKey and try again.';
      }
    } catch (error) {
      console.error('Error listing YubiKeys:', error);
      status = `Error: ${error}`;
    } finally {
      loading = false;
    }
  }
  
  async function generateChallenge() {
    try {
      challenge = await invoke('generate_yubikey_challenge');
      message = 'Challenge generated. Ready for authentication.';
    } catch (error) {
      console.error('Error generating challenge:', error);
      message = `Error: ${error}`;
    }
  }
  
  async function authenticate() {
    if (!selectedYubikey?.serial) {
      message = 'Please select a YubiKey first';
      return;
    }
    
    if (!challenge) {
      await generateChallenge();
    }
    
    loading = true;
    try {
      const result = await invoke('authenticate_with_yubikey', {
        yubikey_serial: selectedYubikey.serial,
        challenge,
      });
      
      if (result) {
        message = 'Authentication successful! ✅';
      } else {
        message = 'Authentication failed! ❌';
      }
    } catch (error) {
      console.error('Authentication error:', error);
      message = `Error: ${error}`;
    } finally {
      loading = false;
    }
  }
  
  async function encryptData() {
    if (!selectedYubikey?.serial) {
      message = 'Please select a YubiKey first';
      return;
    }
    
    if (!textToEncrypt) {
      message = 'Please enter text to encrypt';
      return;
    }
    
    loading = true;
    try {
      encryptedText = await invoke('encrypt_with_yubikey', {
        yubikeySerial: selectedYubikey.serial,
        data: textToEncrypt,
      });
      
      message = 'Text encrypted successfully!';
    } catch (error) {
      console.error('Encryption error:', error);
      message = `Error: ${JSON.stringify(error)}`;
    } finally {
      loading = false;
    }
  }
</script>

<div class="yubikey-container">
  <h2>YubiKey Management</h2>
  
  <div class="section">
    <h3>Connected YubiKeys</h3>
    <div class="action-row">
      <button onclick={listYubikeys} disabled={loading}>
        {loading ? 'Scanning...' : 'Refresh YubiKeys'}
      </button>
      <span class="status">{status}</span>
    </div>
    
    {#if yubikeys.length > 0}
      <div class="yubikey-list">
        <label for="yubikey-select">Select YubiKey:</label>
        <select id="yubikey-select" bind:value={selectedYubikey}>
          {#each yubikeys as key}
            <option value={key}>{key.name} {key.version ? `(${key.version})` : ''}</option>
          {/each}
        </select>
        
        {#if selectedYubikey}
          <div class="yubikey-info">
            <p><strong>Serial:</strong> {selectedYubikey.serial || 'Unknown'}</p>
            <p><strong>Form Factor:</strong> {selectedYubikey.form_factor || 'Unknown'}</p>
            <p><strong>FIPS:</strong> {selectedYubikey.is_fips ? 'Yes' : 'No'}</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
  
  <div class="section">
    <h3>Authentication</h3>
    <div class="action-row">
      <button onclick={authenticate} disabled={loading || !selectedYubikey}>
        {loading ? 'Authenticating...' : 'Authenticate with YubiKey'}
      </button>
    </div>
    <p class="message">{message}</p>
  </div>
  
  <div class="section">
    <h3>Encryption</h3>
    <div class="encrypt-form">
      <label for="encrypt-input">Text to encrypt:</label>
      <textarea id="encrypt-input" rows="3" bind:value={textToEncrypt} placeholder="Enter text to encrypt"></textarea>
      
      <button onclick={encryptData} disabled={loading || !selectedYubikey || !textToEncrypt}>
        {loading ? 'Encrypting...' : 'Encrypt with YubiKey'}
      </button>
      
      {#if encryptedText}
        <div class="result-box">
          <label>Encrypted result:</label>
          <textarea rows="5" readonly value={encryptedText}></textarea>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .yubikey-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
  }
  
  h2 {
    margin-bottom: 20px;
    border-bottom: 1px solid #eee;
    padding-bottom: 10px;
  }
  
  .section {
    margin-bottom: 30px;
    padding: 15px;
    background-color: #f9f9f9;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }
  
  h3 {
    margin-top: 0;
    margin-bottom: 15px;
  }
  
  .action-row {
    display: flex;
    align-items: center;
    margin-bottom: 15px;
  }
  
  button {
    background-color: #007bff;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: background-color 0.2s;
  }
  
  button:hover:not(:disabled) {
    background-color: #0069d9;
  }
  
  button:disabled {
    background-color: #6c757d;
    cursor: not-allowed;
  }
  
  .status {
    margin-left: 15px;
    font-size: 14px;
  }
  
  .message {
    color: #28a745;
    font-weight: 500;
  }
  
  .message:empty {
    display: none;
  }
  
  .yubikey-list {
    margin-top: 15px;
  }
  
  select {
    display: block;
    width: 100%;
    padding: 8px;
    border-radius: 4px;
    border: 1px solid #ced4da;
    margin-top: 5px;
    margin-bottom: 15px;
  }
  
  .yubikey-info {
    background-color: #e9ecef;
    padding: 10px;
    border-radius: 4px;
    margin-top: 10px;
  }
  
  .yubikey-info p {
    margin: 5px 0;
  }
  
  .encrypt-form {
    display: flex;
    flex-direction: column;
  }
  
  textarea {
    width: 100%;
    padding: 8px;
    border-radius: 4px;
    border: 1px solid #ced4da;
    margin: 5px 0 15px 0;
    font-family: inherit;
  }
  
  .result-box {
    margin-top: 15px;
  }
  
  .result-box textarea {
    background-color: #f8f9fa;
  }
</style>
