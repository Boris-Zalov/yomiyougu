<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { 
    Modal, 
    Button, 
    Label, 
    Input, 
    Textarea,
    Spinner,
    Drawer,
  } from "flowbite-svelte";
  import {
    CloseOutline,
    BookmarkSolid,
    HeartSolid,
    HeartOutline,
    CogOutline,
    ChevronLeftOutline,
    ChevronRightOutline,
    ChevronUpOutline,
    ChevronDownOutline,
    CheckCircleSolid,
    CloseCircleSolid,
    PlusOutline,
    EditOutline,
    TrashBinOutline,
  } from "flowbite-svelte-icons";
  import { 
    libraryApi, 
    settingsApi, 
    getEffectiveTheme, 
    getIsAndroid,
    setFullscreen,
    type ThemeMode,
    type Book,
    type BookSettings,
    type Bookmark,
    getPagePath,
  } from "$lib";

  // Route params
  let bookId = $derived(Number(page.params.id));
  
  // Book data
  let book = $state<Book | null>(null);
  let bookSettings = $state<BookSettings | null>(null);
  let bookmarks = $state<Bookmark[]>([]);
  
  // Reading settings
  let readingDirection = $state<"ltr" | "rtl" | "vertical">("rtl");
  let pageDisplayMode = $state<"single" | "double" | "continuous">("single");
  let imageFitMode = $state<"fit_width" | "fit_height" | "fit_screen" | "original">("fit_width");
  
  // Theme
  let isDarkMode = $state(true);
  
  // Reader state
  let currentPage = $state(0);
  let isLoading = $state(true);
  let isImageLoading = $state(false);
  let error = $state<string | null>(null);
  
  // UI state
  let showOverlay = $state(false);
  let showSettingsDrawer = $state(false);
  let showBookmarkDrawer = $state(false);
  let showBookmarkModal = $state(false);
  let showPageJumpModal = $state(false);
  let pageJumpInput = $state("");
  let showToast = $state(false);
  let toastMessage = $state("");
  let toastType = $state<"success" | "error">("success");
  
  // Platform
  let isAndroid = $state(false);
  
  // Bookmark form
  let bookmarkName = $state("");
  let bookmarkDescription = $state("");
  let editingBookmark = $state<Bookmark | null>(null);
  
  // Track pending save operations
  let pendingSave = $state<Promise<void> | null>(null);
  
  // Touch handling for swipe
  let touchStartX = 0;
  let touchStartY = 0;
  let touchEndX = 0;
  let touchEndY = 0;
  
  // Preloaded images
  let preloadedImages = new Set<number>();
  
  // Scroll container for continuous mode
  let scrollContainer: HTMLDivElement | null = $state(null);
  let isScrolling = $state(false);
  let scrollTimeout: ReturnType<typeof setTimeout> | null = null;

  // Computed values
  let totalPages = $derived(book?.total_pages ?? 0);
  let isFavorite = $derived(book?.is_favorite ?? false);
  let isVertical = $derived(readingDirection === "vertical");
  let isContinuous = $derived(pageDisplayMode === "continuous");
  let isDouble = $derived(pageDisplayMode === "double");
  let pageProgress = $derived(totalPages > 0 ? Math.round(((currentPage + 1) / totalPages) * 100) : 0);
  let sortedBookmarks = $derived([...bookmarks].sort((a, b) => a.page - b.page));
  
  // For double page mode: get the second page index (if exists)
  let secondPageIndex = $derived(() => {
    if (!isDouble || currentPage >= totalPages - 1) return null;
    return currentPage + 1;
  });
  
  // Page step for navigation (2 for double mode, 1 otherwise)
  let pageStep = $derived(isDouble ? 2 : 1);
  
  let pageDisplayText = $derived(() => {
    if (isDouble && secondPageIndex() !== null) {
      return `${currentPage + 1}-${currentPage + 2}`;
    }
    return `${currentPage + 1}`;
  });
  
  let isAtLastPage = $derived(() => {
    if (isDouble) {
      return currentPage >= totalPages - 2;
    }
    return currentPage >= totalPages - 1;
  });
  
  // Check if we're at the first page
  let isAtFirstPage = $derived(currentPage === 0);

  // Image fit classes
  let imageFitClass = $derived(() => {
    switch (imageFitMode) {
      case "fit_width":
        return "max-w-full h-auto";
      case "fit_height":
        return "w-auto max-h-full";
      case "fit_screen":
        return "max-w-full max-h-full object-contain";
      case "original":
        return "";
      default:
        return "max-w-full h-auto";
    }
  });

  onMount(async () => {
    isAndroid = getIsAndroid();
    if (isAndroid) {
      setFullscreen(true);
    }
    await loadData();
    document.addEventListener("keydown", handleKeyDown);
  });
  
  onDestroy(() => {
    if (isAndroid) {
      setFullscreen(false);
    }
    if (scrollTimeout) {
      clearTimeout(scrollTimeout);
    }
    document.removeEventListener("keydown", handleKeyDown);
  });

  // Handle scroll for continuous mode
  function handleContinuousScroll() {
    if (!scrollContainer || !isContinuous) return;
    
    isScrolling = true;
    
    if (scrollTimeout) {
      clearTimeout(scrollTimeout);
    }
    
    scrollTimeout = setTimeout(async () => {
      isScrolling = false;
      
      const containerRect = scrollContainer!.getBoundingClientRect();
      const containerCenter = containerRect.top + containerRect.height / 2;
      
      const pageElements = scrollContainer!.querySelectorAll('[data-page-index]');
      let closestPage = 0;
      let closestDistance = Infinity;
      
      pageElements.forEach((el) => {
        const rect = el.getBoundingClientRect();
        const pageCenter = rect.top + rect.height / 2;
        const distance = Math.abs(pageCenter - containerCenter);
        
        if (distance < closestDistance) {
          closestDistance = distance;
          closestPage = parseInt(el.getAttribute('data-page-index') || '0');
        }
      });
      
      if (closestPage !== currentPage) {
        currentPage = closestPage;
        try {
          await libraryApi.updateReadingProgress(bookId, currentPage);
          
          if (currentPage === totalPages - 1 && book) {
            await libraryApi.markAsCompleted(book);
          }
        } catch (e) {
          console.error("Failed to save progress:", e);
        }
      }
    }, 150);
  }

  // Scroll to specific page in continuous mode
  function scrollToPage(pageNum: number) {
    if (!scrollContainer || !isContinuous) return;
    
    const pageElement = scrollContainer.querySelector(`[data-page-index="${pageNum}"]`);
    if (pageElement) {
      pageElement.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  async function loadData() {
    isLoading = true;
    error = null;
    
    try {
      book = await libraryApi.getBook(bookId);
      // Clamp current page to valid range (0 to total_pages - 1)
      currentPage = Math.min(Math.max(0, book.current_page), book.total_pages - 1);
      
      bookSettings = await libraryApi.getBookSettings(bookId);
      bookmarks = await libraryApi.getBookmarks(bookId);
      const settings = await settingsApi.getSettings();
      
      // Theme
      const themeValue = settings.categories
        .find((c) => c.id === "appearance")
        ?.settings.find((s) => s.key === "appearance.theme")
        ?.value as ThemeMode | undefined;
      isDarkMode = getEffectiveTheme(themeValue || "system") === "dark";
      
      const readingCategory = settings.categories.find((c) => c.id === "reading");
      
      const defaultDirection = readingCategory?.settings.find((s) => s.key === "reading.direction")?.value as string ?? "rtl";
      const defaultDisplayMode = readingCategory?.settings.find((s) => s.key === "reading.page_display_mode")?.value as string ?? "single";
      const defaultFitMode = readingCategory?.settings.find((s) => s.key === "reading.image_fit_mode")?.value as string ?? "fit_width";
      
      readingDirection = (bookSettings?.reading_direction ?? defaultDirection) as typeof readingDirection;
      pageDisplayMode = (bookSettings?.page_display_mode ?? defaultDisplayMode) as typeof pageDisplayMode;
      imageFitMode = (bookSettings?.image_fit_mode ?? defaultFitMode) as typeof imageFitMode;
      
      // For double page mode, ensure we start on an even page
      if (pageDisplayMode === "double" && currentPage % 2 !== 0) {
        currentPage = Math.max(0, currentPage - 1);
      }
      
      if (book.reading_status === "unread") {
        await libraryApi.startReading(bookId);
      }
      
      preloadPages();
      
      // Scroll to current page in continuous mode after a short delay (for DOM to update)
      if (pageDisplayMode === "continuous") {
        await tick();
        scrollToPage(currentPage);
      }
      
    } catch (e) {
      console.error("Failed to load book:", e);
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isLoading = false;
    }
  }

  function preloadPages() {
    const pagesToPreload = [currentPage - 1, currentPage + 1, currentPage + 2];
    
    for (const pageNum of pagesToPreload) {
      if (pageNum >= 0 && pageNum < totalPages && !preloadedImages.has(pageNum)) {
        const img = new Image();
        img.src = getPagePath(bookId, pageNum);
        preloadedImages.add(pageNum);
      }
    }
  }

  async function goToPage(pageNum: number) {
    if (pageNum < 0 || pageNum >= totalPages || pageNum === currentPage) return;
    
    // For double page mode, ensure we're on an even page
    if (isDouble && pageNum % 2 !== 0) {
      pageNum = pageNum - 1;
    }
    
    if (pageNum === currentPage) return;
    
    isImageLoading = true;
    currentPage = pageNum;
    const savePromise = (async () => {
      try {
        await libraryApi.updateReadingProgress(bookId, currentPage);
        
        // Check if completed (last page or last spread)
        const isAtEnd = isDouble 
          ? currentPage >= totalPages - 2 
          : currentPage === totalPages - 1;
        if (isAtEnd && book) {
          await libraryApi.markAsCompleted(book);
        }
      } catch (e) {
        console.error("Failed to save progress:", e);
      }
    })();
    
    pendingSave = savePromise;
    await savePromise;
    pendingSave = null;
    
    preloadPages();
  }

  function nextPage() {
    const step = isDouble ? 2 : 1;
    goToPage(currentPage + step);
  }

  function prevPage() {
    const step = isDouble ? 2 : 1;
    goToPage(currentPage - step);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (showBookmarkModal || showSettingsDrawer || showBookmarkDrawer) return;
    
    switch (e.key) {
      case "ArrowLeft":
        if (readingDirection === "rtl") {
          nextPage();
        } else {
          prevPage();
        }
        break;
      case "ArrowRight":
        if (readingDirection === "rtl") {
          prevPage();
        } else {
          nextPage();
        }
        break;
      case "ArrowUp":
        if (isVertical) prevPage();
        break;
      case "ArrowDown":
        if (isVertical) nextPage();
        break;
      case " ":
        e.preventDefault();
        nextPage();
        break;
      case "Escape":
        if (showOverlay) {
          showOverlay = false;
        } else {
          closeReader();
        }
        break;
    }
  }

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.touches[0].clientX;
    touchStartY = e.touches[0].clientY;
  }

  function handleTouchEnd(e: TouchEvent) {
    touchEndX = e.changedTouches[0].clientX;
    touchEndY = e.changedTouches[0].clientY;
    handleSwipe();
  }

  function handleSwipe() {
    const deltaX = touchEndX - touchStartX;
    const deltaY = touchEndY - touchStartY;
    const minSwipeDistance = 50;

    if (isVertical) {
      if (Math.abs(deltaY) > minSwipeDistance && Math.abs(deltaY) > Math.abs(deltaX)) {
        if (deltaY < 0) {
          nextPage();
        } else {
          prevPage();
        }
      }
    } else {
      if (Math.abs(deltaX) > minSwipeDistance && Math.abs(deltaX) > Math.abs(deltaY)) {
        if (readingDirection === "rtl") {
          if (deltaX < 0) nextPage();
          else prevPage();
        } else {
          if (deltaX > 0) prevPage();
          else nextPage();
        }
      }
    }
  }

  function handleReaderClick(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const width = rect.width;
    const height = rect.height;
    
    // Center region (40% of screen) toggles overlay
    const centerXStart = width * 0.3;
    const centerXEnd = width * 0.7;
    const centerYStart = height * 0.3;
    const centerYEnd = height * 0.7;
    
    if (x > centerXStart && x < centerXEnd && y > centerYStart && y < centerYEnd) {
      showOverlay = !showOverlay;
      return;
    }
    
    // Navigation zones for non-vertical modes
    if (!isVertical) {
      if (readingDirection === "rtl") {
        if (x < centerXStart) nextPage();
        else if (x > centerXEnd) prevPage();
      } else {
        if (x < centerXStart) prevPage();
        else if (x > centerXEnd) nextPage();
      }
    } else {
      // Vertical mode: top/bottom zones
      if (y < centerYStart) prevPage();
      else if (y > centerYEnd) nextPage();
    }
  }

  async function toggleFavorite() {
    if (!book) return;
    
    try {
      book = await libraryApi.toggleFavorite(book);
      showToastMessage(book.is_favorite ? "Added to favorites" : "Removed from favorites", "success");
    } catch (e) {
      showToastMessage("Failed to update favorite", "error");
    }
  }

  async function openBookmarkModal() {
    editingBookmark = null;
    bookmarkName = `Page ${currentPage + 1}`;
    bookmarkDescription = "";
    showBookmarkModal = true;
  }

  function openBookmarkDrawer() {
    showBookmarkDrawer = true;
  }

  function editBookmark(bookmark: Bookmark) {
    editingBookmark = bookmark;
    bookmarkName = bookmark.name;
    bookmarkDescription = bookmark.description ?? "";
    showBookmarkModal = true;
  }

  async function saveBookmark() {
    if (!bookmarkName.trim()) return;
    
    try {
      if (editingBookmark) {
        // Update existing bookmark
        const updated = await libraryApi.updateBookmark(
          editingBookmark.id,
          bookmarkName.trim(),
          bookmarkDescription.trim() || undefined
        );
        bookmarks = bookmarks.map(b => b.id === updated.id ? updated : b);
        showToastMessage("Bookmark updated", "success");
      } else {
        // Create new bookmark
        const newBookmark = await libraryApi.createBookmark(
          bookId,
          bookmarkName.trim(),
          currentPage,
          bookmarkDescription.trim() || undefined
        );
        bookmarks = [...bookmarks, newBookmark];
        showToastMessage("Bookmark created", "success");
      }
      showBookmarkModal = false;
      editingBookmark = null;
    } catch (e) {
      showToastMessage(editingBookmark ? "Failed to update bookmark" : "Failed to create bookmark", "error");
    }
  }

  async function deleteBookmark(bookmark: Bookmark) {
    try {
      await libraryApi.deleteBookmark(bookmark.id);
      bookmarks = bookmarks.filter(b => b.id !== bookmark.id);
      showToastMessage("Bookmark deleted", "success");
    } catch (e) {
      showToastMessage("Failed to delete bookmark", "error");
    }
  }

  async function goToBookmark(bookmark: Bookmark) {
    if (isContinuous) {
      scrollToPage(bookmark.page);
      currentPage = bookmark.page;
      try {
        await libraryApi.updateReadingProgress(bookId, currentPage);
      } catch (e) {
        console.error("Failed to save progress:", e);
      }
    } else {
      goToPage(bookmark.page);
    }
    showBookmarkDrawer = false;
    showOverlay = false;
  }

  async function updateBookSetting(key: keyof BookSettings, value: string | boolean) {
    const previousPageDisplayMode = pageDisplayMode;
    const previousReadingDirection = readingDirection;
    
    try {
      const updates: Record<string, string | boolean | null> = {};
      
      if (key === "reading_direction") {
        readingDirection = value as typeof readingDirection;
        updates.readingDirection = value as string;
      } else if (key === "page_display_mode") {
        pageDisplayMode = value as typeof pageDisplayMode;
        updates.pageDisplayMode = value as string;
        
        // Force vertical mode when continuous is selected
        if (value === "continuous" && readingDirection !== "vertical") {
          readingDirection = "vertical";
          updates.readingDirection = "vertical";
        }
      } else if (key === "image_fit_mode") {
        imageFitMode = value as typeof imageFitMode;
        updates.imageFitMode = value as string;
      }
      
      bookSettings = await libraryApi.updateBookSettings(bookId, updates);
      
      // Scroll to current page when switching to continuous/vertical mode
      const switchedToContinuous = previousPageDisplayMode !== "continuous" && pageDisplayMode === "continuous";
      const switchedToVertical = previousReadingDirection !== "vertical" && readingDirection === "vertical";
      
      if (switchedToContinuous || switchedToVertical) {
        await tick();
        scrollToPage(currentPage);
      }
    } catch (e) {
      showToastMessage("Failed to update setting", "error");
    }
  }

  function showToastMessage(message: string, type: "success" | "error") {
    toastMessage = message;
    toastType = type;
    showToast = true;
    setTimeout(() => { showToast = false; }, 3000);
  }

  function openPageJumpModal() {
    pageJumpInput = String(currentPage + 1);
    showPageJumpModal = true;
  }

  function handlePageJump() {
    const pageNum = parseInt(pageJumpInput, 10);
    if (isNaN(pageNum) || pageNum < 1 || pageNum > totalPages) {
      showToastMessage(`Please enter a valid page (1-${totalPages})`, "error");
      return;
    }
    
    const targetPage = pageNum - 1; // Convert to 0-indexed
    if (isContinuous) {
      scrollToPage(targetPage);
      currentPage = targetPage;
      libraryApi.updateReadingProgress(bookId, currentPage).catch(console.error);
    } else {
      goToPage(targetPage);
    }
    showPageJumpModal = false;
    showOverlay = false;
  }

  async function closeReader() {
    // Wait for any pending save operation to complete
    if (pendingSave) {
      await pendingSave;
    }
    goto("/library");
  }

  function onImageLoad() {
    isImageLoading = false;
  }
</script>

{#if isLoading}
  <div 
    class="h-full w-full flex items-center justify-center"
    class:bg-black={isDarkMode}
    class:bg-gray-100={!isDarkMode}
  >
    <Spinner size="12" />
  </div>
{:else if error}
  <div 
    class="h-full w-full flex flex-col items-center justify-center p-4"
    class:bg-black={isDarkMode}
    class:text-white={isDarkMode}
    class:bg-gray-100={!isDarkMode}
    class:text-gray-900={!isDarkMode}
  >
    <CloseCircleSolid class="w-16 h-16 text-red-500 mb-4" />
    <p class="text-lg mb-4">Failed to load book</p>
    <p class="text-sm opacity-70 mb-8">{error}</p>
    <Button onclick={closeReader}>Back to Library</Button>
  </div>
{:else}
  <!-- Main Reader - keyboard handling is done via document event listener in onMount -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
  <div 
    class="h-full w-full relative select-none"
    class:overflow-hidden={!isContinuous}
    class:overflow-y-auto={isContinuous}
    class:bg-black={isDarkMode}
    class:bg-gray-100={!isDarkMode}
    onclick={isContinuous ? undefined : handleReaderClick}
    onkeydown={() => {}}
    ontouchstart={isContinuous ? undefined : handleTouchStart}
    ontouchend={isContinuous ? undefined : handleTouchEnd}
    onscroll={isContinuous ? handleContinuousScroll : undefined}
    bind:this={scrollContainer}
    role="application"
    aria-label="Comic reader"
  >
    <!-- Continuous Mode: All pages in a vertical scrollable container -->
    {#if isContinuous}
      <div 
        class="flex flex-col items-center gap-1 pb-16"
        onclick={() => showOverlay = !showOverlay}
        onkeydown={() => {}}
        role="button"
        tabindex="0"
      >
        {#each Array(totalPages) as _, pageIndex}
          <div 
            data-page-index={pageIndex}
            class="flex items-center justify-center w-full"
          >
            <img 
              src={getPagePath(bookId, pageIndex)}
              alt="Page {pageIndex + 1}"
              class={imageFitClass()}
              draggable="false"
              loading="lazy"
            />
          </div>
        {/each}
      </div>
    <!-- Double Page Mode: Two pages side by side -->
    {:else if isDouble}
      <div class="h-full w-full flex items-center justify-center">
        {#if isImageLoading}
          <div class="absolute inset-0 flex items-center justify-center">
            <Spinner size="8" />
          </div>
        {/if}
        <div class="h-full flex items-center justify-center gap-1 {readingDirection === 'rtl' ? 'flex-row-reverse' : 'flex-row'}">
          <!-- First page (left in LTR, right in RTL) -->
          {#key currentPage}
          <img 
            src={getPagePath(bookId, currentPage)}
            alt="Page {currentPage + 1}"
            class="{imageFitClass()} max-w-[50vw]"
            onload={onImageLoad}
            onerror={onImageLoad}
            draggable="false"
          />
          {/key}
          <!-- Second page (if exists) -->
          {#if secondPageIndex() !== null}
            {#key secondPageIndex()}
            <img 
              src={getPagePath(bookId, secondPageIndex()!)}
              alt="Page {secondPageIndex()! + 1}"
              class="{imageFitClass()} max-w-[50vw]"
              draggable="false"
            />
            {/key}
          {/if}
        </div>
      </div>
    <!-- Single Page Mode -->
    {:else}
      <div class="h-full w-full flex items-center justify-center">
        {#if isImageLoading}
          <div class="absolute inset-0 flex items-center justify-center">
            <Spinner size="8" />
          </div>
        {/if}
        {#key currentPage}
        <img 
          src={getPagePath(bookId, currentPage)}
          alt="Page {currentPage + 1}"
          class={imageFitClass()}
          onload={onImageLoad}
          onerror={onImageLoad}
          draggable="false"
        />
        {/key}
      </div>
    {/if}

    <!-- Navigation Hints (shown briefly or on hover) - only for single/double non-vertical -->
    {#if !isVertical && !isContinuous && !showOverlay}
      <div class="absolute inset-y-0 left-0 w-[30%] flex items-center justify-start pl-4 opacity-0 hover:opacity-30 transition-opacity pointer-events-none">
        {#if readingDirection === "rtl"}
          <ChevronRightOutline class="w-12 h-12 text-white drop-shadow-lg" />
        {:else}
          <ChevronLeftOutline class="w-12 h-12 text-white drop-shadow-lg" />
        {/if}
      </div>
      <div class="absolute inset-y-0 right-0 w-[30%] flex items-center justify-end pr-4 opacity-0 hover:opacity-30 transition-opacity pointer-events-none">
        {#if readingDirection === "rtl"}
          <ChevronLeftOutline class="w-12 h-12 text-white drop-shadow-lg" />
        {:else}
          <ChevronRightOutline class="w-12 h-12 text-white drop-shadow-lg" />
        {/if}
      </div>
    {/if}

  </div>

    <!-- Bottom Progress Bar (fixed position so it doesn't scroll with content) -->
    <div class="fixed bottom-0 left-0 right-0 h-1 bg-black/30 z-30 {readingDirection === 'rtl' ? 'flex justify-end' : ''}">
      <div 
        class="h-full bg-primary-500 transition-all duration-300"
        style="width: {pageProgress}%"
      ></div>
    </div>

    <!-- Overlay UI (fixed position so it doesn't scroll with content) -->
    {#if showOverlay}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="fixed inset-0 bg-black/40 transition-opacity z-40" role="presentation" onclick={(e) => { e.stopPropagation(); showOverlay = false; }}>
        <!-- Top Bar (title+actions on desktop, page info on Android) -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div 
          class="fixed top-0 left-0 right-0 p-4 flex items-center justify-between z-50 {isDarkMode ? 'bg-gray-900/90' : 'bg-white/90'}"
          onclick={(e) => e.stopPropagation()}
          role="toolbar"
          aria-label="Reader toolbar"
          tabindex="0"
        >
          {#if isAndroid}
            <!-- Android: Page info centered at top -->
            <div class="flex-1"></div>
            <div class="flex items-center gap-2 mt-7">
              {#if isVertical}
                <button 
                  class="p-2 rounded-full hover:bg-black/20 transition-colors disabled:opacity-30"
                  onclick={(e) => { e.stopPropagation(); prevPage(); }}
                  disabled={isAtFirstPage}
                  aria-label="Previous page"
                >
                  <ChevronUpOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-900'}" />
                </button>
                <button 
                  class="text-sm font-medium px-2 hover:underline {isDarkMode ? 'text-white' : 'text-gray-900'}"
                  onclick={(e) => { e.stopPropagation(); openPageJumpModal(); }}
                >
                  {pageDisplayText()} / {totalPages}
                </button>
                <button 
                  class="p-2 rounded-full hover:bg-black/20 transition-colors disabled:opacity-30"
                  onclick={(e) => { e.stopPropagation(); nextPage(); }}
                  disabled={isAtLastPage()}
                  aria-label="Next page"
                >
                  <ChevronDownOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-900'}" />
                </button>
              {:else}
                <button 
                  class="p-2 rounded-full hover:bg-black/20 transition-colors disabled:opacity-30"
                  onclick={(e) => { e.stopPropagation(); readingDirection === "rtl" ? nextPage() : prevPage(); }}
                  disabled={readingDirection === "rtl" ? isAtLastPage() : isAtFirstPage}
                  aria-label={readingDirection === "rtl" ? "Next page" : "Previous page"}
                >
                  <ChevronLeftOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-900'}" />
                </button>
                <button 
                  class="text-sm font-medium px-2 hover:underline {isDarkMode ? 'text-white' : 'text-gray-900'}"
                  onclick={(e) => { e.stopPropagation(); openPageJumpModal(); }}
                >
                  {pageDisplayText()} / {totalPages}
                </button>
                <button 
                  class="p-2 rounded-full hover:bg-black/20 transition-colors disabled:opacity-30"
                  onclick={(e) => { e.stopPropagation(); readingDirection === "rtl" ? prevPage() : nextPage(); }}
                  disabled={readingDirection === "rtl" ? isAtFirstPage : isAtLastPage()}
                  aria-label={readingDirection === "rtl" ? "Previous page" : "Next page"}
                >
                  <ChevronRightOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-900'}" />
                </button>
              {/if}
            </div>
            <div class="flex-1"></div>
          {:else}
            <!-- Desktop: Title and actions at top -->
            <div class="flex items-center gap-4">
              <span class:text-white={isDarkMode} class:text-gray-900={!isDarkMode} class="font-medium truncate max-w-[200px] md:max-w-none">
                {book?.title ?? "Loading..."}
              </span>
            </div>
            
            <div class="flex items-center gap-2">
            <!-- Favorite -->
            <button 
              onclick={toggleFavorite}
              class="p-2 rounded-lg hover:bg-black/20 transition-colors"
              aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
            >
              {#if isFavorite}
                <HeartSolid class="w-6 h-6 text-red-500" />
              {:else}
                <HeartOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
              {/if}
            </button>

            <!-- Bookmark -->
            <button 
              onclick={openBookmarkDrawer}
              class="p-2 rounded-lg hover:bg-black/20 transition-colors"
              aria-label="Bookmarks"
            >
              <BookmarkSolid class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
            </button>

            <!-- Settings -->
            <button 
              onclick={() => showSettingsDrawer = true}
              class="p-2 rounded-lg hover:bg-black/20 transition-colors"
              aria-label="Reading settings"
            >
              <CogOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
            </button>

            <!-- Close -->
            <button 
              onclick={closeReader}
              class="p-2 rounded-lg hover:bg-black/20 transition-colors"
              aria-label="Close reader"
            >
              <CloseOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
            </button>
            </div>
          {/if}
        </div>

        <!-- Bottom Bar - Page Info (on desktop) / Title+Actions (on Android) -->
        {#if isAndroid}
          <!-- Android: Title and actions at bottom -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div 
            class="fixed bottom-0 left-0 right-0 p-4 flex items-center justify-between z-50 {isDarkMode ? 'bg-gray-900/90' : 'bg-white/90'}"
            onclick={(e) => e.stopPropagation()}
            role="toolbar"
            aria-label="Reader toolbar"
            tabindex="0"
          >
            <div class="flex items-center gap-4">
              <span class:text-white={isDarkMode} class:text-gray-900={!isDarkMode} class="font-medium truncate max-w-[200px]">
                {book?.title ?? "Loading..."}
              </span>
            </div>
            
            <div class="flex items-center gap-2">
              <!-- Favorite -->
              <button 
                onclick={toggleFavorite}
                class="p-2 rounded-lg hover:bg-black/20 transition-colors"
                aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
              >
                {#if isFavorite}
                  <HeartSolid class="w-6 h-6 text-red-500" />
                {:else}
                  <HeartOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
                {/if}
              </button>

              <!-- Bookmark -->
              <button 
                onclick={openBookmarkDrawer}
                class="p-2 rounded-lg hover:bg-black/20 transition-colors"
                aria-label="Bookmarks"
              >
                <BookmarkSolid class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
              </button>

              <!-- Settings -->
              <button 
                onclick={() => showSettingsDrawer = true}
                class="p-2 rounded-lg hover:bg-black/20 transition-colors"
                aria-label="Reading settings"
              >
                <CogOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
              </button>

              <!-- Close -->
              <button 
                onclick={closeReader}
                class="p-2 rounded-lg hover:bg-black/20 transition-colors"
                aria-label="Close reader"
              >
                <CloseOutline class="w-6 h-6 {isDarkMode ? 'text-white' : 'text-gray-700'}" />
              </button>
            </div>
          </div>
        {:else}
          <!-- Desktop: Page info at bottom center (clickable to jump to page) -->
          <button 
            class="fixed bottom-4 left-1/2 -translate-x-1/2 px-4 py-2 rounded-full z-50 hover:ring-2 hover:ring-primary-500 transition-all {isDarkMode ? 'bg-gray-900/90 text-white' : 'bg-white/90 text-gray-900'}"
            onclick={(e) => { e.stopPropagation(); openPageJumpModal(); }}
            aria-label="Jump to page"
          >
            <span class="text-sm font-medium">
              {pageDisplayText()} / {totalPages}
            </span>
          </button>
        {/if}

        <!-- Navigation Buttons (desktop only - on Android, they're in the bottom bar) -->
        {#if !isAndroid && !isContinuous}
          {#if isVertical}
            <button 
              class="fixed top-1/2 left-4 -translate-y-1/2 p-3 rounded-full bg-black/50 hover:bg-black/70 transition-colors disabled:opacity-30 z-50"
              onclick={(e) => { e.stopPropagation(); prevPage(); }}
              disabled={isAtFirstPage}
              aria-label="Previous page"
            >
              <ChevronUpOutline class="w-6 h-6 text-white" />
            </button>
            <button 
              class="fixed top-1/2 right-4 -translate-y-1/2 p-3 rounded-full bg-black/50 hover:bg-black/70 transition-colors disabled:opacity-30 z-50"
              onclick={(e) => { e.stopPropagation(); nextPage(); }}
              disabled={isAtLastPage()}
              aria-label="Next page"
            >
              <ChevronDownOutline class="w-6 h-6 text-white" />
            </button>
          {:else}
            <button 
              class="fixed top-1/2 left-4 -translate-y-1/2 p-3 rounded-full bg-black/50 hover:bg-black/70 transition-colors disabled:opacity-30 z-50"
              onclick={(e) => { e.stopPropagation(); readingDirection === "rtl" ? nextPage() : prevPage(); }}
              disabled={readingDirection === "rtl" ? isAtLastPage() : isAtFirstPage}
              aria-label={readingDirection === "rtl" ? "Next page" : "Previous page"}
            >
              <ChevronLeftOutline class="w-6 h-6 text-white" />
            </button>
            <button 
              class="fixed top-1/2 right-4 -translate-y-1/2 p-3 rounded-full bg-black/50 hover:bg-black/70 transition-colors disabled:opacity-30 z-50"
              onclick={(e) => { e.stopPropagation(); readingDirection === "rtl" ? prevPage() : nextPage(); }}
              disabled={readingDirection === "rtl" ? isAtFirstPage : isAtLastPage()}
              aria-label={readingDirection === "rtl" ? "Previous page" : "Next page"}
            >
              <ChevronRightOutline class="w-6 h-6 text-white" />
            </button>
          {/if}
        {/if}
      </div>
    {/if}
{/if}

<!-- Bookmark Drawer -->
<Drawer bind:open={showBookmarkDrawer} placement="right" aria-labelledby="bookmark-drawer-label">
  <div class="flex items-center justify-between mb-4 mt-11">
    <h5 id="bookmark-drawer-label" class="inline-flex items-center text-base font-semibold text-gray-500 dark:text-gray-400">
      <BookmarkSolid class="me-2.5 h-5 w-5" />
      Bookmarks
    </h5>
    <Button size="sm" onclick={openBookmarkModal}>
      <PlusOutline class="w-4 h-4 me-1" />
      Add
    </Button>
  </div>
  {#if sortedBookmarks.length === 0}
    <p class="text-sm text-gray-500 dark:text-gray-400">
      No bookmarks yet. Add one to save your place.
    </p>
  {:else}
    <div class="space-y-2 overflow-y-auto max-h-[calc(100vh-180px)]">
      {#each sortedBookmarks as bookmark (bookmark.id)}
        <div class="p-3 rounded-lg bg-gray-100 dark:bg-gray-700">
          <button
            onclick={() => goToBookmark(bookmark)}
            class="w-full text-left hover:opacity-80 transition-opacity"
          >
            <div class="font-medium text-gray-900 dark:text-white">{bookmark.name}</div>
            <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">Page {bookmark.page + 1}</div>
            {#if bookmark.description}
              <div class="text-sm text-gray-600 dark:text-gray-300 mt-1 line-clamp-2">{bookmark.description}</div>
            {/if}
          </button>
          <div class="flex gap-2 mt-2 pt-2 border-t border-gray-200 dark:border-gray-600">
            <button
              onclick={() => editBookmark(bookmark)}
              class="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-600 dark:text-gray-300 transition-colors"
              aria-label="Edit bookmark"
            >
              <EditOutline class="w-3.5 h-3.5" />
              Edit
            </button>
            <button
              onclick={() => deleteBookmark(bookmark)}
              class="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400 transition-colors"
              aria-label="Delete bookmark"
            >
              <TrashBinOutline class="w-3.5 h-3.5" />
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</Drawer>

<!-- Settings Drawer -->
<Drawer bind:open={showSettingsDrawer} placement="right" aria-labelledby="settings-drawer-label">
  <div class="mb-4 mt-11">
    <h5 id="settings-drawer-label" class="inline-flex items-center text-base font-semibold text-gray-500 dark:text-gray-400">
      <CogOutline class="me-2.5 h-5 w-5" />
      Reading Settings
    </h5>
  </div>
  
  <div class="space-y-6">
    <!-- Reading Direction -->
    <div class:opacity-50={isContinuous}>
      <span class="text-sm font-medium block mb-2 text-gray-900 dark:text-white">
        Reading Direction
        {#if isContinuous}
          <span class="text-xs text-gray-500 dark:text-gray-400">(forced vertical)</span>
        {/if}
      </span>
      <div class="grid grid-cols-3 gap-2">
        <button 
          onclick={() => updateBookSetting("reading_direction", "ltr")}
          class="px-3 py-2 text-xs rounded transition-colors {readingDirection === 'ltr' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
          disabled={isContinuous}
        >
          LTR
        </button>
        <button 
          onclick={() => updateBookSetting("reading_direction", "rtl")}
          class="px-3 py-2 text-xs rounded transition-colors {readingDirection === 'rtl' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
          disabled={isContinuous}
        >
          RTL
        </button>
        <button 
          onclick={() => updateBookSetting("reading_direction", "vertical")}
          class="px-3 py-2 text-xs rounded transition-colors {readingDirection === 'vertical' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
          disabled={isContinuous}
        >
          Vertical
        </button>
      </div>
    </div>

    <!-- Page Display Mode -->
    <div>
      <span class="text-sm font-medium block mb-2 text-gray-900 dark:text-white">Page Display</span>
      <div class="grid grid-cols-3 gap-2">
        <button 
          onclick={() => updateBookSetting("page_display_mode", "single")}
          class="px-3 py-2 text-xs rounded transition-colors {pageDisplayMode === 'single' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Single
        </button>
        <button 
          onclick={() => updateBookSetting("page_display_mode", "double")}
          class="px-3 py-2 text-xs rounded transition-colors {pageDisplayMode === 'double' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Double
        </button>
        <button 
          onclick={() => updateBookSetting("page_display_mode", "continuous")}
          class="px-3 py-2 text-xs rounded transition-colors {pageDisplayMode === 'continuous' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Continuous
        </button>
      </div>
    </div>

    <!-- Image Fit Mode -->
    <div>
      <span class="text-sm font-medium block mb-2 text-gray-900 dark:text-white">Image Fit</span>
      <div class="grid grid-cols-2 gap-2">
        <button 
          onclick={() => updateBookSetting("image_fit_mode", "fit_width")}
          class="px-3 py-2 text-xs rounded transition-colors {imageFitMode === 'fit_width' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Fit Width
        </button>
        <button 
          onclick={() => updateBookSetting("image_fit_mode", "fit_height")}
          class="px-3 py-2 text-xs rounded transition-colors {imageFitMode === 'fit_height' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Fit Height
        </button>
        <button 
          onclick={() => updateBookSetting("image_fit_mode", "fit_screen")}
          class="px-3 py-2 text-xs rounded transition-colors {imageFitMode === 'fit_screen' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Fit Screen
        </button>
        <button 
          onclick={() => updateBookSetting("image_fit_mode", "original")}
          class="px-3 py-2 text-xs rounded transition-colors {imageFitMode === 'original' ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'}"
        >
          Original
        </button>
      </div>
    </div>
  </div>
</Drawer>

<!-- Bookmark Modal -->
<Modal bind:open={showBookmarkModal} size="sm" autoclose={false}>
  <div class="space-y-4">
    <h3 class="text-lg font-semibold">{editingBookmark ? 'Edit Bookmark' : 'Create Bookmark'}</h3>
    <div>
      <Label for="bookmark-name" class="mb-2">Name</Label>
      <Input 
        id="bookmark-name" 
        bind:value={bookmarkName} 
        placeholder="Bookmark name"
      />
    </div>
    <div>
      <Label for="bookmark-desc" class="mb-2">Description (optional)</Label>
      <Textarea 
        id="bookmark-desc" 
        class="resize-none w-full"
        bind:value={bookmarkDescription} 
        placeholder="Add a note..."
        rows={3}
      />
    </div>
    <div class="text-sm text-gray-500">
      Page: {editingBookmark ? editingBookmark.page + 1 : currentPage + 1}
    </div>
    <div class="flex gap-2 justify-end mt-4">
      <Button onclick={() => { showBookmarkModal = false; editingBookmark = null; }} color="alternative">Cancel</Button>
      <Button onclick={saveBookmark} disabled={!bookmarkName.trim()}>{editingBookmark ? 'Save' : 'Create'}</Button>
    </div>
  </div>
</Modal>

<!-- Page Jump Modal -->
<Modal bind:open={showPageJumpModal} size="xs" autoclose={false}>
  <div class="space-y-4">
    <h3 class="text-lg font-semibold">Go to Page</h3>
    <div>
      <Label for="page-jump-input" class="mb-2">Page Number (1-{totalPages})</Label>
      <Input 
        id="page-jump-input" 
        type="number"
        min="1"
        max={totalPages}
        bind:value={pageJumpInput} 
        placeholder="Enter page number"
        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handlePageJump(); }}
      />
    </div>
    <div class="flex gap-2 justify-end mt-4">
      <Button onclick={() => showPageJumpModal = false} color="alternative">Cancel</Button>
      <Button onclick={handlePageJump}>Go</Button>
    </div>
  </div>
</Modal>

<!-- Toast -->
{#if showToast}
  <div class="fixed bottom-20 left-1/2 -translate-x-1/2 z-50">
    <div class="flex items-center gap-2 px-4 py-3 rounded-lg shadow-lg {toastType === 'success' ? 'bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100' : 'bg-red-100 text-red-800 dark:bg-red-800 dark:text-red-100'}">
      {#if toastType === "success"}
        <CheckCircleSolid class="w-5 h-5" />
      {:else}
        <CloseCircleSolid class="w-5 h-5" />
      {/if}
      <span>{toastMessage}</span>
    </div>
  </div>
{/if}
