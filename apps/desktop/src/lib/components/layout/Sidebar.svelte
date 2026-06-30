<script lang="ts">
  import { page } from "$app/stores";
  import {
    Blueprint,
    ClockCounterClockwise,
    DesktopTower,
    FlowArrow,
    GearSix,
    SquaresFour,
  } from "phosphor-svelte";
  import type { Component } from "svelte";

  const links: { href: string; label: string; icon: Component }[] = [
    { href: "/", label: "Dashboard", icon: SquaresFour },
    { href: "/flows", label: "Fluxos", icon: FlowArrow },
    { href: "/blueprints", label: "Blueprints", icon: Blueprint },
    { href: "/admin", label: "Admin", icon: DesktopTower },
    { href: "/history", label: "Histórico", icon: ClockCounterClockwise },
    { href: "/settings", label: "Configurações", icon: GearSix },
  ];

  function isActive(href: string, pathname: string): boolean {
    if (href === "/") return pathname === "/";
    return pathname === href || pathname.startsWith(`${href}/`);
  }
</script>

<nav class="sidebar" aria-label="Navegação principal">
  <div class="brand">
    <span class="brand-mark" aria-hidden="true">
      <FlowArrow size={16} weight="bold" />
    </span>
    <div>
      <strong>TidyFlow</strong>
      <small>v2 preview</small>
    </div>
  </div>

  <ul role="list">
    {#each links as link}
      {@const active = isActive(link.href, $page.url.pathname)}
      <li>
        <a
          href={link.href}
          class:active
          aria-current={active ? "page" : undefined}
        >
          <link.icon size={17} weight={active ? "fill" : "regular"} aria-hidden="true" />
          <span>{link.label}</span>
        </a>
      </li>
    {/each}
  </ul>
</nav>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: var(--space-4) var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    min-height: 100vh;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-2);
  }

  .brand-mark {
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-md);
    display: grid;
    place-items: center;
    background: var(--accent);
    color: white;
    flex-shrink: 0;
  }

  .brand strong {
    font-size: var(--text-sm);
    font-weight: 650;
    letter-spacing: -0.01em;
  }

  .brand small {
    display: block;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  a {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0.5rem 0.6rem;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: 500;
    transition: background 120ms ease, color 120ms ease;
  }

  a:hover {
    background: var(--surface-muted);
    color: var(--text-primary);
  }

  a.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
