<script>
    import { Button } from "flowbite-svelte";
    import { goto } from "$app/navigation";
    import appState from "$lib/AppState.svelte";
    import {invoke} from "@tauri-apps/api/core";
    
    let logOut = async () => {
        invoke("log_out").then(() => {
            appState.refreshAuthState();
            goto("/account/log_in");
        }).catch((e) => {
            console.error(e);
        });
    };
</script>
<nav class="bg-white border-b border-gray-200 px-4 py-2.5 dark:bg-gray-800 dark:border-gray-700 fixed left-0 right-0 top-0 z-50">
  <div class="flex flex-wrap justify-between items-center">
    <div class="flex justify-start items-center">
      <button
        data-drawer-target="drawer-navigation"
        data-drawer-toggle="drawer-navigation"
        aria-controls="drawer-navigation"
        class="p-2 mr-2 text-gray-600 rounded-lg cursor-pointer md:hidden hover:text-gray-900 hover:bg-gray-100 focus:bg-gray-100 dark:focus:bg-gray-700 focus:ring-2 focus:ring-gray-100 dark:focus:ring-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
      >
        <svg aria-hidden="true" class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
          <path fill-rule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h6a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"></path>
        </svg>
        <span class="sr-only">Toggle sidebar</span>
      </button>
      <a href="https://flowbite.com" class="flex items-center justify-between mr-4">
        <img src="/svelte.svg" class="mr-3 h-8" alt="Flowbite Logo" />
        <span class="self-center text-2xl font-semibold whitespace-nowrap dark:text-white">Vault</span>
      </a>
      <!-- <form action="#" method="GET" class="hidden md:block md:pl-2">
        <label for="topbar-search" class="sr-only">Search</label>
        <div class="relative md:w-64 md:w-96">
          <div class="flex absolute inset-y-0 left-0 items-center pl-3 pointer-events-none">
            <svg class="w-5 h-5 text-gray-500 dark:text-gray-400" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"></path>
            </svg>
          </div>
          <input type="text" name="email" id="topbar-search" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-500 focus:border-primary-500 block w-full pl-10 p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="Search" />
        </div>
      </form> -->
    </div>
    <div class="flex items-center lg:order-2">
      <Button onclick={logOut} color="light">Exit</Button>
      <!-- Notifications button and dropdown -->
      <!-- Apps button and dropdown -->
      <!-- User menu button and dropdown -->
      <!-- ... Keep all the existing buttons and dropdowns ... -->
    </div>
  </div>
</nav>
