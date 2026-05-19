// ================================================================
// FH Language Combo Tool — Frontend Application
// ================================================================

(function () {
  'use strict';

  // ── Tauri API ──────────────────────────────────────────────────

  function invoke(cmd, args) {
    return window.__TAURI__.core.invoke(cmd, args);
  }

  // ── State ──────────────────────────────────────────────────────

  let games = [];
  let selectedGame = null;
  let languagePacks = [];
  let currentPlan = null;
  let currentStatus = null;
  let backups = [];
  let selectedBackup = null;

  // ── Voice Language Whitelist ────────────────────────────────────
  // Only these language codes have voice audio banks (FH5 list).
  // Case-insensitive comparison is used against scanned pack codes.

  const VOICE_CODES = new Set([
    'EN', 'JP', 'CHS', 'CHT', 'BR', 'DE', 'ES', 'FR', 'IT', 'KO', 'MX'
  ]);

  // ── Status Labels ──────────────────────────────────────────────

  const STATUS_LABELS = {
    applied: '已应用',
    reverted: '已还原',
    modified: '已修改',
    none: '未配置'
  };

  const STATUS_BADGE_CLASS = {
    applied: 'badge-success',
    reverted: 'badge-warn',
    modified: 'badge-danger',
    none: 'badge-muted'
  };

  // ── DOM References ─────────────────────────────────────────────

  const $ = (sel) => document.querySelector(sel);
  const $$ = (sel) => document.querySelectorAll(sel);

  // Pages
  const pageDisclaimer = $('#page-disclaimer');
  const pageMain = $('#page-main');
  const pageConfirm = $('#page-confirm');
  const pageResultSuccess = $('#page-result-success');
  const pageResultError = $('#page-result-error');
  const pageRestore = $('#page-restore');

  // Disclaimer
  const btnAcceptDisclaimer = $('#btn-accept-disclaimer');

  // Main — Games
  const gameList = $('#game-list');
  const gameEmpty = $('#game-empty');
  const btnToggleManual = $('#btn-toggle-manual');
  const manualInputArea = $('#manual-input-area');
  const manualGameId = $('#manual-game-id');
  const manualPath = $('#manual-path');
  const btnValidatePath = $('#btn-validate-path');
  const manualError = $('#manual-error');

  // Main — Status
  const statusBadge = $('#status-badge');

  // Main — Language
  const languageSection = $('#language-section');
  const selectVoice = $('#select-voice');
  const selectText = $('#select-text');
  const effectPreview = $('#effect-preview');
  const effectDesc = $('#effect-desc');
  const effectDetail = $('#effect-detail');
  const currentStatusBar = $('#current-status');
  const statusText = $('#status-text');

  // Main — Actions
  const actionSection = $('#action-section');
  const btnApply = $('#btn-apply');
  const btnRestore = $('#btn-restore');

  // Confirm
  const confirmGame = $('#confirm-game');
  const confirmVoice = $('#confirm-voice');
  const confirmText = $('#confirm-text');
  const confirmDesc = $('#confirm-desc');
  const btnConfirmBack = $('#btn-confirm-back');
  const btnConfirmApply = $('#btn-confirm-apply');

  // Results
  const resultSuccessMsg = $('#result-success-msg');
  const resultSuccessDetail = $('#result-success-detail');
  const btnSuccessBack = $('#btn-success-back');
  const resultErrorMsg = $('#result-error-msg');
  const resultErrorDetail = $('#result-error-detail');
  const btnErrorBack = $('#btn-error-back');

  // Restore
  const backupListEl = $('#backup-list');
  const backupEmpty = $('#backup-empty');
  const restoreConfirmArea = $('#restore-confirm-area');
  const restoreTime = $('#restore-time');
  const restoreVoice = $('#restore-voice');
  const restoreText = $('#restore-text');
  const btnRestoreBack = $('#btn-restore-back');
  const btnRestoreConfirm = $('#btn-restore-confirm');

  // Loading
  const loadingOverlay = $('#loading-overlay');
  const loadingText = $('#loading-text');

  // ── Page Navigation ────────────────────────────────────────────

  function showPage(page) {
    $$('.page').forEach((p) => p.classList.remove('active'));
    page.classList.add('active');
  }

  // ── Loading ────────────────────────────────────────────────────

  function showLoading(text) {
    loadingText.textContent = text || '处理中...';
    loadingOverlay.style.display = '';
  }

  function hideLoading() {
    loadingOverlay.style.display = 'none';
  }

  // ── Error Helpers ──────────────────────────────────────────────

  function extractErrorMessage(err) {
    if (typeof err === 'string') return err;
    if (err && err.message) return err.message;
    return String(err);
  }

  // ── Disclaimer ─────────────────────────────────────────────────

  function initDisclaimer() {
    const checks = $$('.disclaimer-check');
    const updateBtn = () => {
      const allChecked = Array.from(checks).every((c) => c.checked);
      btnAcceptDisclaimer.disabled = !allChecked;
    };
    checks.forEach((cb) => {
      cb.addEventListener('change', updateBtn);
      cb.closest('.checkbox-item').addEventListener('click', () => {
        setTimeout(updateBtn, 0);
      });
    });

    btnAcceptDisclaimer.addEventListener('click', () => {
      localStorage.setItem('fhlct_disclaimer_accepted', '1');
      showPage(pageMain);
      detectGames();
    });
  }

  // ── Game Detection ─────────────────────────────────────────────

  async function detectGames() {
    gameEmpty.innerHTML = '<div class="spinner"></div><span>正在检测已安装的游戏...</span>';
    gameEmpty.style.display = '';

    try {
      const detected = await invoke('detect_games');
      games = detected || [];
    } catch (err) {
      games = [];
      gameEmpty.innerHTML = '<span class="text-muted">检测失败：' + escapeHtml(extractErrorMessage(err)) + '</span>';
      return;
    }

    renderGameList();

    if (games.length > 0) {
      selectGame(games[0]);
    }
  }

  function renderGameList() {
    // Clear existing game cards but keep the empty state element
    const existingCards = gameList.querySelectorAll('.game-card');
    existingCards.forEach((c) => c.remove());

    if (games.length === 0) {
      gameEmpty.innerHTML = '<span class="text-muted">未检测到已安装的 Forza Horizon 游戏。请尝试手动添加。</span>';
      gameEmpty.style.display = '';
      return;
    }

    gameEmpty.style.display = 'none';

    games.forEach((game, index) => {
      const card = document.createElement('div');
      card.className = 'game-card';
      card.dataset.index = index;

      const iconLabel = game.gameId === 'fh5' ? 'FH5' : 'FH6';
      card.innerHTML =
        '<div class="game-icon">' + escapeHtml(iconLabel) + '</div>' +
        '<div class="game-info">' +
          '<div class="game-name">' + escapeHtml(game.displayName) + '</div>' +
          '<div class="game-path" title="' + escapeHtml(game.rootPath) + '">' + escapeHtml(game.rootPath) + '</div>' +
        '</div>' +
        '<span class="game-channel">' + escapeHtml(game.channel) + '</span>';

      card.addEventListener('click', () => selectGame(game));
      gameList.insertBefore(card, gameEmpty);
    });
  }

  async function selectGame(game) {
    selectedGame = game;

    // Update visual selection
    gameList.querySelectorAll('.game-card').forEach((card) => {
      const idx = parseInt(card.dataset.index, 10);
      card.classList.toggle('selected', games[idx] === game);
    });

    // Show language section and actions
    languageSection.style.display = '';
    actionSection.style.display = '';

    // Reset dropdowns
    selectVoice.innerHTML = '<option value="">-- 选择语音语言 --</option>';
    selectText.innerHTML = '<option value="">-- 选择文字语言 --</option>';
    effectPreview.style.display = 'none';
    btnApply.disabled = true;

    // Scan language packs and get status in parallel
    try {
      const [packs, status] = await Promise.all([
        invoke('scan_language_packs', { resourcePath: game.resourcePath }),
        invoke('get_status', { gameId: game.gameId, resourcePath: game.resourcePath })
      ]);

      languagePacks = packs || [];
      currentStatus = status;

      populateLanguageDropdowns();
      updateStatusDisplay();
    } catch (err) {
      languagePacks = [];
      currentStatus = null;
      statusBadge.style.display = 'none';
      currentStatusBar.style.display = 'none';
      showInlineError('扫描语言包失败：' + extractErrorMessage(err));
    }
  }

  function showInlineError(msg) {
    // Show error in the effect preview area
    effectPreview.style.display = '';
    effectDesc.textContent = msg;
    effectDesc.style.color = 'var(--danger)';
    effectDetail.textContent = '';
  }

  // ── Language Dropdowns ─────────────────────────────────────────

  function populateLanguageDropdowns() {
    // Voice: only languages in the whitelist
    selectVoice.innerHTML = '<option value="">-- 选择语音语言 --</option>';
    selectText.innerHTML = '<option value="">-- 选择文字语言 --</option>';

    languagePacks.forEach((pack) => {
      // Text dropdown: all languages
      const textOpt = document.createElement('option');
      textOpt.value = pack.code;
      textOpt.textContent = pack.displayName + ' (' + pack.code + ')';
      selectText.appendChild(textOpt);

      // Voice dropdown: only if in whitelist
      if (VOICE_CODES.has(pack.code.toUpperCase())) {
        const voiceOpt = document.createElement('option');
        voiceOpt.value = pack.code;
        voiceOpt.textContent = pack.displayName + ' (' + pack.code + ')';
        selectVoice.appendChild(voiceOpt);
      }
    });

    // If current status has applied languages, pre-select them
    if (currentStatus && currentStatus.state === 'applied') {
      preselectLanguage(selectVoice, currentStatus.voiceLanguage);
      preselectLanguage(selectText, currentStatus.textLanguage);
      updateEffectPreview();
    }
  }

  function preselectLanguage(selectEl, langCode) {
    if (!langCode) return;
    const upper = langCode.toUpperCase();
    for (let i = 0; i < selectEl.options.length; i++) {
      if (selectEl.options[i].value.toUpperCase() === upper) {
        selectEl.selectedIndex = i;
        return;
      }
    }
  }

  function updateEffectPreview() {
    const voice = selectVoice.value;
    const text = selectText.value;

    effectDesc.style.color = '';

    if (!voice || !text) {
      effectPreview.style.display = 'none';
      btnApply.disabled = true;
      return;
    }

    if (voice.toUpperCase() === text.toUpperCase()) {
      effectPreview.style.display = '';
      effectDesc.textContent = '语音语言和文字语言相同，无需修改。';
      effectDesc.style.color = 'var(--warn)';
      effectDetail.textContent = '';
      btnApply.disabled = true;
      return;
    }

    const voicePack = languagePacks.find((p) => p.code === voice);
    const textPack = languagePacks.find((p) => p.code === text);
    const voiceName = voicePack ? voicePack.displayName : voice;
    const textName = textPack ? textPack.displayName : text;

    effectPreview.style.display = '';
    effectDesc.textContent = '游戏将使用 ' + voiceName + ' 语音 + ' + textName + ' 文字';
    effectDetail.textContent =
      '操作：将 ' + text + '.zip（文字）复制覆盖到 ' + voice + '.zip（语音），' +
      '原始 ' + voice + '.zip 将被自动备份。';

    btnApply.disabled = false;
  }

  // ── Status Display ─────────────────────────────────────────────

  function updateStatusDisplay() {
    if (!currentStatus) {
      statusBadge.style.display = 'none';
      currentStatusBar.style.display = 'none';
      return;
    }

    const state = currentStatus.state;
    const label = STATUS_LABELS[state] || state;
    const badgeClass = STATUS_BADGE_CLASS[state] || 'badge-muted';

    statusBadge.textContent = label;
    statusBadge.className = 'badge ' + badgeClass;
    statusBadge.style.display = '';

    if (state !== 'none' && currentStatus.voiceLanguage && currentStatus.textLanguage) {
      currentStatusBar.style.display = '';
      const voicePack = languagePacks.find((p) => p.code.toUpperCase() === currentStatus.voiceLanguage.toUpperCase());
      const textPack = languagePacks.find((p) => p.code.toUpperCase() === currentStatus.textLanguage.toUpperCase());
      const voiceName = voicePack ? voicePack.displayName : currentStatus.voiceLanguage;
      const textName = textPack ? textPack.displayName : currentStatus.textLanguage;
      statusText.textContent =
        label + ' — 语音：' + voiceName + '，文字：' + textName;
      if (currentStatus.lastApplied) {
        statusText.textContent += '（' + formatDateTime(currentStatus.lastApplied) + '）';
      }
    } else {
      currentStatusBar.style.display = 'none';
    }
  }

  // ── Apply Flow ─────────────────────────────────────────────────

  async function startApplyFlow() {
    if (!selectedGame || !selectVoice.value || !selectText.value) return;

    showLoading('检查游戏运行状态...');

    try {
      const running = await invoke('check_game_running', { gameId: selectedGame.gameId });
      hideLoading();

      if (running) {
        alert('游戏正在运行中，请先关闭游戏后再进行操作。');
        return;
      }
    } catch (err) {
      hideLoading();
      // If check fails, warn but allow to proceed
      if (!confirm('无法检测游戏运行状态（' + extractErrorMessage(err) + '）。\n是否仍然继续？')) {
        return;
      }
    }

    // Build plan info for confirmation
    const voicePack = languagePacks.find((p) => p.code === selectVoice.value);
    const textPack = languagePacks.find((p) => p.code === selectText.value);

    currentPlan = {
      gameId: selectedGame.gameId,
      voiceLang: selectVoice.value,
      textLang: selectText.value,
      resourcePath: selectedGame.resourcePath,
      manifestPath: selectedGame.manifestPath || null,
      voiceDisplayName: voicePack ? voicePack.displayName : selectVoice.value,
      textDisplayName: textPack ? textPack.displayName : selectText.value
    };

    // Populate confirm page
    confirmGame.textContent = selectedGame.displayName;
    confirmVoice.textContent = currentPlan.voiceDisplayName + ' (' + currentPlan.voiceLang + ')';
    confirmText.textContent = currentPlan.textDisplayName + ' (' + currentPlan.textLang + ')';
    let desc = '将 ' + currentPlan.textLang + '.zip（文字语言包）复制覆盖到 ' +
      currentPlan.voiceLang + '.zip（语音语言包）。';
    if (currentPlan.manifestPath) {
      desc += '\n同时自动设置 Steam 游戏启动语言为 ' + currentPlan.voiceDisplayName + '，无需手动切换。';
    } else {
      desc += '\n请手动在 Steam 游戏属性中将语言设置为 ' + currentPlan.voiceDisplayName + '。';
    }
    confirmDesc.textContent = desc;

    // Reset confirm checkboxes
    $$('.confirm-check').forEach((cb) => { cb.checked = false; });
    btnConfirmApply.disabled = true;

    showPage(pageConfirm);
  }

  async function executeApply() {
    if (!currentPlan) return;

    showLoading('正在应用配置...');

    try {
      const result = await invoke('apply_config', {
        gameId: currentPlan.gameId,
        voiceLang: currentPlan.voiceLang,
        textLang: currentPlan.textLang,
        resourcePath: currentPlan.resourcePath,
        manifestPath: currentPlan.manifestPath
      });

      hideLoading();

      if (result.success) {
        let msg = '已成功将 ' + currentPlan.textDisplayName + ' 文字应用到 ' +
          currentPlan.voiceDisplayName + ' 语音包。';
        if (result.steamLanguageSet) {
          msg += '\nSteam 启动语言已自动设置为 ' + currentPlan.voiceDisplayName + '。';
        }
        if (result.steamLanguageWarning) {
          msg += '\n' + result.steamLanguageWarning;
        }
        resultSuccessMsg.textContent = msg;

        let detail = '';
        if (result.backupPath) {
          detail += '备份路径：' + result.backupPath;
        }
        resultSuccessDetail.textContent = detail;

        showPage(pageResultSuccess);
      } else {
        resultErrorMsg.textContent = result.message || '应用配置时发生未知错误。';
        let detail = '';
        if (result.rolledBack) {
          detail += '已自动回滚到原始状态。\n';
        }
        if (result.backupPath) {
          detail += '备份路径：' + result.backupPath;
        }
        resultErrorDetail.textContent = detail;

        showPage(pageResultError);
      }
    } catch (err) {
      hideLoading();
      resultErrorMsg.textContent = extractErrorMessage(err);
      resultErrorDetail.textContent = '';
      showPage(pageResultError);
    }
  }

  // ── Restore Flow ───────────────────────────────────────────────

  async function startRestoreFlow() {
    if (!selectedGame) return;

    showLoading('加载备份列表...');
    selectedBackup = null;
    restoreConfirmArea.style.display = 'none';
    btnRestoreConfirm.style.display = 'none';

    try {
      const list = await invoke('list_backups', { gameId: selectedGame.gameId });
      backups = list || [];
      hideLoading();
    } catch (err) {
      hideLoading();
      backups = [];
      alert('加载备份列表失败：' + extractErrorMessage(err));
      return;
    }

    renderBackupList();
    showPage(pageRestore);
  }

  function renderBackupList() {
    backupListEl.innerHTML = '';

    if (backups.length === 0) {
      backupListEl.innerHTML = '<div class="no-backups-message">暂无备份记录。</div>';
      return;
    }

    backups.forEach((backup, index) => {
      const item = document.createElement('div');
      item.className = 'backup-item' + (backup.valid ? '' : ' invalid');
      item.dataset.index = index;

      const voiceName = getDisplayNameForCode(backup.voiceLanguage);
      const textName = getDisplayNameForCode(backup.textLanguage);

      item.innerHTML =
        '<div class="backup-info">' +
          '<div class="backup-time">' + escapeHtml(formatDateTime(backup.createdAt)) + '</div>' +
          '<div class="backup-langs">语音：' + escapeHtml(voiceName) +
            ' · 文字：' + escapeHtml(textName) + '</div>' +
        '</div>' +
        '<div class="backup-badge">' +
          (backup.valid
            ? '<span class="badge badge-success">有效</span>'
            : '<span class="badge badge-danger">无效</span>') +
        '</div>';

      if (backup.valid) {
        item.addEventListener('click', () => selectBackupItem(backup, index));
      }

      backupListEl.appendChild(item);
    });
  }

  function selectBackupItem(backup, index) {
    selectedBackup = backup;

    backupListEl.querySelectorAll('.backup-item').forEach((item) => {
      item.classList.toggle('selected', parseInt(item.dataset.index, 10) === index);
    });

    const voiceName = getDisplayNameForCode(backup.voiceLanguage);
    const textName = getDisplayNameForCode(backup.textLanguage);

    restoreTime.textContent = formatDateTime(backup.createdAt);
    restoreVoice.textContent = voiceName + ' (' + backup.voiceLanguage + ')';
    restoreText.textContent = textName + ' (' + backup.textLanguage + ')';

    restoreConfirmArea.style.display = '';
    btnRestoreConfirm.style.display = '';
  }

  async function executeRestore() {
    if (!selectedBackup) return;

    showLoading('正在恢复备份...');

    try {
      // Check if game is running first
      try {
        const running = await invoke('check_game_running', { gameId: selectedGame.gameId });
        if (running) {
          hideLoading();
          alert('游戏正在运行中，请先关闭游戏后再进行恢复操作。');
          return;
        }
      } catch (_) {
        // Ignore check failure, proceed with restore
      }

      const result = await invoke('restore_backup', { backupPath: selectedBackup.path });
      hideLoading();

      if (result.success) {
        resultSuccessMsg.textContent = '备份恢复成功！游戏资源文件已还原到原始状态。';
        resultSuccessDetail.textContent = result.message || '';
        showPage(pageResultSuccess);
      } else {
        resultErrorMsg.textContent = result.message || '恢复备份时发生未知错误。';
        resultErrorDetail.textContent = '';
        showPage(pageResultError);
      }
    } catch (err) {
      hideLoading();
      resultErrorMsg.textContent = extractErrorMessage(err);
      resultErrorDetail.textContent = '';
      showPage(pageResultError);
    }
  }

  // ── Manual Directory Validation ────────────────────────────────

  async function validateManualPath() {
    const path = manualPath.value.trim();
    const gameId = manualGameId.value;

    if (!path) {
      showManualError('请输入游戏根目录路径。');
      return;
    }

    manualError.style.display = 'none';
    btnValidatePath.disabled = true;
    btnValidatePath.textContent = '验证中...';

    try {
      const profile = await invoke('validate_game_directory', { path: path, gameId: gameId });

      // Check if this game is already in the list
      const existing = games.findIndex(
        (g) => g.gameId === profile.gameId && g.rootPath === profile.rootPath
      );
      if (existing === -1) {
        games.push(profile);
      }

      renderGameList();
      selectGame(profile);

      // Hide manual input
      manualInputArea.style.display = 'none';
      manualPath.value = '';
      manualError.style.display = 'none';
    } catch (err) {
      showManualError(extractErrorMessage(err));
    } finally {
      btnValidatePath.disabled = false;
      btnValidatePath.textContent = '验证';
    }
  }

  function showManualError(msg) {
    manualError.textContent = msg;
    manualError.style.display = '';
  }

  // ── Utility Functions ──────────────────────────────────────────

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }

  function formatDateTime(isoStr) {
    if (!isoStr) return '未知';
    try {
      const date = new Date(isoStr);
      if (isNaN(date.getTime())) return isoStr;
      const y = date.getFullYear();
      const m = String(date.getMonth() + 1).padStart(2, '0');
      const d = String(date.getDate()).padStart(2, '0');
      const h = String(date.getHours()).padStart(2, '0');
      const min = String(date.getMinutes()).padStart(2, '0');
      const s = String(date.getSeconds()).padStart(2, '0');
      return y + '-' + m + '-' + d + ' ' + h + ':' + min + ':' + s;
    } catch (_) {
      return isoStr;
    }
  }

  function getDisplayNameForCode(code) {
    if (!code) return '未知';
    // Try to find in current language packs first
    const pack = languagePacks.find((p) => p.code.toUpperCase() === code.toUpperCase());
    if (pack) return pack.displayName;

    // Fallback mapping
    const map = {
      EN: 'English', GB: 'English (UK)', JP: '日本語',
      CHS: '简体中文', CHT: '繁體中文', FR: 'Français',
      DE: 'Deutsch', ES: 'Español', MX: 'Español (MX)',
      IT: 'Italiano', PT: 'Português', BR: 'Português (BR)',
      KO: '한국어', RU: 'Русский', PL: 'Polski',
      NL: 'Nederlands', TR: 'Türkçe', DK: 'Dansk',
      SV: 'Svenska', NO: 'Norsk', FI: 'Suomi',
      CZ: 'Čeština', HU: 'Magyar', EL: 'Ελληνικά'
    };
    return map[code.toUpperCase()] || code;
  }

  // ── Event Binding ──────────────────────────────────────────────

  function bindEvents() {
    // Disclaimer
    initDisclaimer();

    // Manual path toggle
    btnToggleManual.addEventListener('click', () => {
      const visible = manualInputArea.style.display !== 'none';
      manualInputArea.style.display = visible ? 'none' : '';
      btnToggleManual.textContent = visible ? '+ 手动添加游戏目录' : '- 收起手动添加';
    });

    // Manual path validation
    btnValidatePath.addEventListener('click', validateManualPath);
    manualPath.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') validateManualPath();
    });

    // Language selection changes
    selectVoice.addEventListener('change', updateEffectPreview);
    selectText.addEventListener('change', updateEffectPreview);

    // Apply button
    btnApply.addEventListener('click', startApplyFlow);

    // Restore button
    btnRestore.addEventListener('click', startRestoreFlow);

    // Confirm page checkboxes
    const confirmChecks = $$('.confirm-check');
    const updateConfirmBtn = () => {
      const allChecked = Array.from(confirmChecks).every((c) => c.checked);
      btnConfirmApply.disabled = !allChecked;
    };
    confirmChecks.forEach((cb) => {
      cb.addEventListener('change', updateConfirmBtn);
      cb.closest('.checkbox-item').addEventListener('click', () => {
        setTimeout(updateConfirmBtn, 0);
      });
    });

    // Confirm page buttons
    btnConfirmBack.addEventListener('click', () => showPage(pageMain));
    btnConfirmApply.addEventListener('click', executeApply);

    // Result page buttons
    btnSuccessBack.addEventListener('click', () => {
      showPage(pageMain);
      // Refresh status after successful apply/restore
      if (selectedGame) {
        refreshStatus();
      }
    });

    btnErrorBack.addEventListener('click', () => {
      showPage(pageMain);
      if (selectedGame) {
        refreshStatus();
      }
    });

    // Restore page buttons
    btnRestoreBack.addEventListener('click', () => showPage(pageMain));
    btnRestoreConfirm.addEventListener('click', executeRestore);
  }

  // ── Status Refresh ─────────────────────────────────────────────

  async function refreshStatus() {
    if (!selectedGame) return;
    try {
      currentStatus = await invoke('get_status', {
        gameId: selectedGame.gameId,
        resourcePath: selectedGame.resourcePath
      });
      updateStatusDisplay();
    } catch (_) {
      // Silently ignore refresh errors
    }
  }

  // ── Init ───────────────────────────────────────────────────────

  function init() {
    bindEvents();

    // Check if disclaimer was already accepted
    if (localStorage.getItem('fhlct_disclaimer_accepted') === '1') {
      showPage(pageMain);
      detectGames();
    } else {
      showPage(pageDisclaimer);
    }
  }

  // Wait for DOM to be ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
