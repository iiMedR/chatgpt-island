import { ChangeDetectionStrategy, Component, ElementRef, viewChild } from '@angular/core';
import { DownloadButton } from './components/download-button';
import { Icon } from './components/icon';

import { CHECKSUM_URL, REPOSITORY } from './release-download';

@Component({
  selector: 'app-root',
  imports: [DownloadButton, Icon],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App {
  readonly repository = REPOSITORY;
  readonly checksumUrl = CHECKSUM_URL;

  private readonly privacy = viewChild.required<ElementRef<HTMLDialogElement>>('privacy');
  openPrivacy(): void {
    this.privacy().nativeElement.showModal();
  }
  closePrivacy(): void {
    this.privacy().nativeElement.close();
  }
}
