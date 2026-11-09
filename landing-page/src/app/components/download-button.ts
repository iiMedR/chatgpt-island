import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { Icon } from './icon';
import { INSTALLER_URL } from '../release-download';

@Component({
  selector: 'app-download-button',
  imports: [Icon],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `<a class="download" [class.compact]="compact()" [href]="installerUrl">
    @if (!compact()) {
      <app-icon name="windows" />
    }
    <span>{{ compact() ? 'Download' : 'Download for Windows' }}</span>
  </a>`,
  styles: `
    :host {
      display: inline-flex;
    }
    .download {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 12px;
      min-height: 56px;
      padding: 0 26px;
      border: 1px solid #161616;
      border-radius: 13px;
      background: #151515;
      color: #fff;
      font: inherit;
      font-size: 20px;
      font-weight: 680;
      letter-spacing: -0.025em;
      text-decoration: none;
      transition:
        background 0.15s,
        transform 0.15s;
      white-space: nowrap;
    }
    .download app-icon {
      font-size: 26px;
    }
    .download:hover {
      background: #333;
      transform: translateY(-1px);
    }
    .download:active {
      transform: translateY(0);
    }
    .download.compact {
      font-size: 15px;
      min-height: 36px;
      padding: 0 16px;
      border-radius: 10px;
    }
    .download:focus-visible {
      outline: 3px solid #5b88b2;
      outline-offset: 4px;
    }
    @media (max-width: 600px) {
      :host:not(.nav-download) {
        justify-content: center;
      }
      .download:not(.compact) {
        font-size: 18px;
        min-height: 54px;
        padding-inline: 22px;
      }
      .download.compact {
        font-size: 13px;
        padding-inline: 13px;
      }
    }
  `,
})
export class DownloadButton {
  readonly compact = input(false);
  readonly installerUrl = INSTALLER_URL;
}
