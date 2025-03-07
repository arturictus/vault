<script lang="ts">
  import { twMerge } from 'tailwind-merge';
  import Frame from '../utils/Frame.svelte';
  import CloseButton from '../utils/CloseButton.svelte';
  import focusTrap from '../utils/focusTrap';
  import type { Dismissable, SizeType } from '$lib/types';
  import type { ModalPlacementType } from '../types';
  import type { ComponentProps } from 'svelte';

  // Define props using Svelte 5 $props rune
  interface ModalProps extends ComponentProps<Frame>, Dismissable {
    open?: boolean;
    title?: string;
    size?: SizeType;
    placement?: ModalPlacementType;
    autoclose?: boolean;
    outsideclose?: boolean;
    backdropClass?: string;
    classBackdrop?: string;
    dialogClass?: string;
    classDialog?: string;
    defaultClass?: string;
    headerClass?: string;
    classHeader?: string;
    bodyClass?: string;
    classBody?: string;
    footerClass?: string;
    classFooter?: string;
  }
  
  // Using Svelte 5 props rune
  const {
    open = $bindable(false),
    title = '',
    size = 'md',
    color = 'default',
    placement = 'center',
    autoclose = false,
    outsideclose = false,
    dismissable = true,
    backdropClass = 'fixed inset-0 z-40 bg-gray-900 bg-black/50 dark:bg-black/80',
    classBackdrop = undefined,
    dialogClass = 'fixed top-0 start-0 end-0 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex',
    classDialog = undefined,
    defaultClass = 'relative flex flex-col mx-auto',
    headerClass = 'flex justify-between items-center p-4 md:p-5 rounded-t-lg',
    classHeader = undefined,
    bodyClass = 'p-4 md:p-5 space-y-4 flex-1 overflow-y-auto overscroll-contain',
    classBody = undefined,
    footerClass = 'flex items-center p-4 md:p-5 space-x-3 rtl:space-x-reverse rounded-b-lg',
    classFooter = undefined,
    ...restProps
  } = $props();

  // Using Svelte 5 state runes for event tracking
  let isOpen = $state(open);
  
  // Event listeners for open/close events
  $effect(() => {
    // Dispatch open/close events
    if (isOpen) {
      const event = new CustomEvent('open');
      document.dispatchEvent(event);
    } else {
      const event = new CustomEvent('close');
      document.dispatchEvent(event);
    }
  });
  
  // Keep state and prop in sync
  $effect(() => {
    isOpen = open;
  });

  function prepareFocus(node: HTMLElement) {
    const walker = document.createTreeWalker(node, NodeFilter.SHOW_ELEMENT);
    let n: Node | null;
    while ((n = walker.nextNode())) {
      if (n instanceof HTMLElement) {
        const el = n as HTMLElement;
        const [x, y] = isScrollable(el);
        if (x || y) el.tabIndex = 0;
      }
    }
    node.focus();
  }

  const getPlacementClasses = (placement: ModalPlacementType) => {
    switch (placement) {
      // top
      case 'top-left':
        return ['justify-start', 'items-start'];
      case 'top-center':
        return ['justify-center', 'items-start'];
      case 'top-right':
        return ['justify-end', 'items-start'];

      // center
      case 'center-left':
        return ['justify-start', 'items-center'];
      case 'center':
        return ['justify-center', 'items-center'];
      case 'center-right':
        return ['justify-end', 'items-center'];

      // bottom
      case 'bottom-left':
        return ['justify-start', 'items-end'];
      case 'bottom-center':
        return ['justify-center', 'items-end'];
      case 'bottom-right':
        return ['justify-end', 'items-end'];

      default:
        return ['justify-center', 'items-center'];
    }
  };

  const sizes = {
    xs: 'max-w-md',
    sm: 'max-w-lg',
    md: 'max-w-2xl',
    lg: 'max-w-4xl',
    xl: 'max-w-7xl'
  };

  const onAutoClose = (e: MouseEvent) => {
    const target: Element = e.target as Element;
    if (autoclose && target?.tagName === 'BUTTON') hide(e); // close on any button click
  };

  const onOutsideClose = (e: MouseEvent) => {
    const target: Element = e.target as Element;
    if (outsideclose && target === e.currentTarget) hide(e); // close on click outside
  };

  const hide = (e: Event) => {
    e.preventDefault();
    isOpen = false;
  };

  const isScrollable = (e: HTMLElement): boolean[] => [e.scrollWidth > e.clientWidth && ['scroll', 'auto'].indexOf(getComputedStyle(e).overflowX) >= 0, e.scrollHeight > e.clientHeight && ['scroll', 'auto'].indexOf(getComputedStyle(e).overflowY) >= 0];

  function handleKeys(e: KeyboardEvent) {
    if (e.key === 'Escape' && dismissable) return hide(e);
  }

  // Using Svelte 5 derived state
  const backdropCls = $derived(twMerge(backdropClass, classBackdrop));
  const dialogCls = $derived(twMerge(dialogClass, classDialog, getPlacementClasses(placement).join(' ')));
  const frameCls = $derived(twMerge(defaultClass, 'w-full divide-y', restProps.class));
  const headerCls = $derived(twMerge(headerClass, classHeader));
  const bodyCls = $derived(twMerge(bodyClass, classBody));
  const footerCls = $derived(twMerge(footerClass, classFooter));
</script>

{#if isOpen}
  <!-- backdrop -->
  <div class={backdropCls}></div>
  <!-- dialog -->
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div on:keydown={handleKeys} on:wheel|preventDefault|nonpassive use:prepareFocus use:focusTrap on:click={onAutoClose} on:mousedown={onOutsideClose} class={dialogCls} tabindex="-1" aria-modal="true" role="dialog">
    <div class="flex relative {sizes[size]} w-full max-h-full">
      <!-- Modal content -->
      <Frame rounded shadow {...restProps} class={frameCls} {color}>
        <!-- Modal header -->
        {#if $$slots.header || title}
          <Frame class={headerCls} {color}>
            <slot name="header">
              <h3 class="text-xl font-semibold {color === 'default' ? '' : 'text-gray-900 dark:text-white'} p-0">
                {title}
              </h3>
            </slot>
            {#if dismissable}<CloseButton name="Close modal" {color} on:click={hide} />{/if}
          </Frame>
        {/if}
        <!-- Modal body -->
        <div class={bodyCls} role="document" on:keydown|stopPropagation={handleKeys} on:wheel|stopPropagation|passive>
          {#if dismissable && !$$slots.header && !title}
            <CloseButton name="Close modal" class="absolute top-3 end-2.5" {color} on:click={hide} />
          {/if}
          <slot></slot>
        </div>
        <!-- Modal footer -->
        {#if $$slots.footer}
          <Frame class={footerCls} {color}>
            <slot name="footer"></slot>
          </Frame>
        {/if}
      </Frame>
    </div>
  </div>
{/if}

<!--
@component
[Go to docs](https://flowbite-svelte.com/)
## Props
@prop export let open: $$Props['open'] = false;
@prop export let title: $$Props['title'] = '';
@prop export let size: NonNullable<$$Props['size']> = 'md';
@prop export let color: $$Props['color'] = 'default';
@prop export let placement: NonNullable<$$Props['placement']> = 'center';
@prop export let autoclose: $$Props['autoclose'] = false;
@prop export let outsideclose: $$Props['outsideclose'] = false;
@prop export let dismissable: $$Props['dismissable'] = true;
@prop export let backdropClass: $$Props['backdropClass'] = 'fixed inset-0 z-40 bg-gray-900 bg-black/50 dark:bg-black/80';
@prop export let classBackdrop: $$Props['classBackdrop'] = undefined;
@prop export let dialogClass: $$Props['dialogClass'] = 'fixed top-0 start-0 end-0 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex';
@prop export let classDialog: $$Props['classDialog'] = undefined;
@prop export let defaultClass: $$Props['defaultClass'] = 'relative flex flex-col mx-auto';
@prop export let headerClass: $$Props['headerClass'] = 'flex justify-between items-center p-4 md:p-5 rounded-t-lg';
@prop export let classHeader: $$Props['classHeader'] = undefined;
@prop export let bodyClass: $$Props['bodyClass'] = 'p-4 md:p-5 space-y-4 flex-1 overflow-y-auto overscroll-contain';
@prop export let classBody: $$Props['classBody'] = undefined;
@prop export let footerClass: $$Props['footerClass'] = 'flex items-center p-4 md:p-5 space-x-3 rtl:space-x-reverse rounded-b-lg';
@prop export let classFooter: $$Props['classFooter'] = undefined;
-->