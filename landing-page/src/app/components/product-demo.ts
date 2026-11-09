import { ChangeDetectionStrategy, Component } from '@angular/core';
import { Icon } from './icon';

@Component({
  selector: 'app-product-demo',
  imports: [Icon],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `<figure
    class="demo-canvas"
    aria-label="Illustration of selected text becoming an unsent ChatGPT draft"
  >
    <div class="source-window">
      <p>
        Modern applications are often built from multiple, loosely coupled components. One common
        pattern for keeping those components flexible is dependency injection.
      </p>
      <p>
        <mark
          >Dependency injection lets an object receive what it needs from another component.</mark
        >
      </p>
      <p>This makes the code easier to test and maintain.</p>
    </div>
    <div class="connection" aria-hidden="true">
      <div class="keybinding mini">
        <kbd>Alt</kbd><span>+</span><kbd>Shift</kbd><span>+</span><kbd>E</kbd>
      </div>
      <svg class="flow-arrow" viewBox="0 0 156 24" fill="none">
        <path
          d="M2 12h149m-9-9 10 9-10 9"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </div>
    <div class="chat-window">
      <div class="chat-brand">ChatGPT</div>
      <div class="composer">
        <p>Regarding this selected text:</p>
        <p>Dependency injection lets an object receive what it needs from another component.</p>
        <p>My question: Explain this simply.</p>
        <div class="composer-bottom">
          <span class="send-example" aria-hidden="true"><app-icon name="send" /></span>
        </div>
      </div>
    </div>
    <figcaption class="visually-hidden">
      Illustrative preview, not a live ChatGPT session. The installed app prepares an editable
      draft; you choose when to send.
    </figcaption>
  </figure>`,
  styleUrl: './product-demo.css',
})
export class ProductDemo {}
