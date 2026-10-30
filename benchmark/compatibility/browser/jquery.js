import $ from 'jquery';
import { check } from './check.js';

globalThis.runWorkload = async () => {
  document.body.innerHTML =
    '<main><ul></ul><form><input name="name" value="Ada"><input type="checkbox" name="active" value="yes" checked></form></main>';
  const events = [];
  $('ul').on('click', 'button', function () {
    events.push($(this).text());
    $(this).closest('li').remove();
  });
  $('ul').append('<li><button>one</button></li><li><button>two</button></li>');
  $('li').first().find('button').trigger('click');
  const result = { events, remaining: $('li').text(), form: $('form').serialize() };
  check(result.events.join() === 'one', 'delegated click must run exactly once');
  check(result.remaining === 'two', 'click must remove the correct item');
  check(result.form === 'name=Ada&active=yes', 'successful form controls must serialize');
  return result;
};
