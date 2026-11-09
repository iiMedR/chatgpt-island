import { ChangeDetectionStrategy, Component, input } from '@angular/core';

@Component({
  selector: 'app-icon',
  changeDetection: ChangeDetectionStrategy.OnPush,
  styles:
    ':host {display:inline-flex; width:1em; height:1em; flex:none} svg {width:100%;height:100%;display:block}',
  template: `<svg viewBox="0 0 24 24" fill="none" aria-hidden="true" focusable="false">
    @switch (name()) {
      @case ('windows') {
        <path
          fill="currentColor"
          d="M2 4.8 10.4 3.6v8H2V4.8Zm9.7-1.4L22 2v9.6H11.7V3.4ZM2 12.8h8.4v8L2 19.6v-6.8Zm9.7 0H22V22l-10.3-1.4v-7.8Z"
        />
      }
      @case ('external') {
        <path
          d="M6 18 18 6M7 6h11v11"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      }
      @case ('close') {
        <path
          d="m6 6 12 12M6 18 18 6"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
        />
      }
      @case ('send') {
        <path d="m3 3 19 9-19 9 3.8-9L3 3Z" fill="currentColor" />
        <path d="M7 12h11" stroke="var(--send-cutout, white)" stroke-width="1.5" />
      }
    }
  </svg>`,
})
export class Icon {
  readonly name = input.required<'windows' | 'external' | 'close' | 'send'>();
}
