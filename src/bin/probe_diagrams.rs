// GENERATED diagram smoke runner (TEXT mode).
use callplot::api::render;
use callplot::constants::MODE_TEXT;
use callplot::errors::Ex;
use callplot::errors::error_name;
use callplot::jcompat::U;

pub fn report(r: Result<U, Ex>, idx: i64) {
    match r {
        Ok(u) => {
            println!("OK {} {}", idx, u.len());
            for c in u {
                println!("{}", c);
            }
        }
        Err(e) => {
            println!("ERR {} {}", idx, error_name(e.kind));
        }
    }
}

pub fn main() {
    report(render("opt/title/POST /orders on the fictional ShopAPI\nopt/messageNumbering/on\nopt/columnPitch/18\nguy/c/Mobile app\nguy/g/API gateway\nguy/o/Order service\nguy/d/Postgres\nguy/k/Redis cache\nguy/q/Event bus^Kafka\nguy/a/AI model^fraud + upsell\nguy/p/Payment API^external\nguy/w/Warehouse^external\nc->g/POST /orders (JWT)\ng->g/verify JWT, rate limit\ng->o/createOrder(cart)/order id\no->k/GET cart:42/miss\no->d/SELECT cart, prices, stock/rows\no->a/score order risk/risk 0.07\n!!model claude, 1.2 s, 410 tokens\no->d/BEGIN, INSERT order, INSERT items, UPDATE stock/ok\no->p/charge card (idempotency key)/402 declined\n!The first charge fails. The service retries once with a backup method.\no->p/charge card, retry/200 paid\no->d/UPDATE order SET status = PAID/ok\no->q/publish order.paid^async\nq->w/order.paid/ack\nw->q/shipment.created^async\nq->o/shipment.created\no->d/UPDATE order SET shipment = ...\no->k/DEL cart:42\no->g/201 Created\ng->c/201 + order id\nnote/q/Other consumers get the event:^email, analytics, loyalty points.\n".to_string(), MODE_TEXT), 0);
    report(render("opt/title/callplot: what happens inside one run\nopt/messageNumbering/on\nopt/columnPitch/17\nguy/u/User^shell\nguy/cli/bin/callplot.js^CLI\nguy/ix/render()^index.js\nguy/ps/parser.js\nguy/fl/flow.js^Flow + layout\nguy/rd/render/*^renderer\nguy/ir/ir.js^IR\nguy/h/html /^mermaid\nguy/jc/jcompat.js\nu->cli/callplot -svg fred > fred.svg\ncli->cli/scan flags (Java rules)^pick mode + options\ncli->cli/read file bytes (fs)^decode as UTF-8\ncli->ix/render(text, mode, ctx, options)\nix->ps/parse(text)\nps->jc/javaTrim, javaSplit\njc->ps/clean tokens\nps->ix/Flow with players[], messages[]\nix->fl/layout(): options, columns, numbering\nfl->jc/fmtD, idiv, parseJavaInt\njc->fl/Java-exact numbers\nfl->ix/positions and widths ready\nnote/ix/Compatible outputs read Flow.^New outputs read the IR.\nix->rd/draw(Flow, ctx)^static switch on mode\nrd->rd/text, pic, svg, vml, graffle, table\nrd->ix/string\nix->ir/buildModel(Flow)^only for -mermaid, -plantuml, -html\nir->h/events e1..eN + srcLine\nh->ix/string\nix->cli/output string\ncli->u/stdout bytes, exit code 0\n".to_string(), MODE_TEXT), 1);
    report(render("opt/title/Use callplot: from command line to browser\nopt/messageNumbering/on\nopt/columnPitch/16\nguy/e/Engineer\nguy/t/Terminal\nguy/c/callplot\nguy/f/Files\nguy/b/Browser\ne->f/write calls.msd^guy, arrows, ! comments\ne->t/callplot -html calls.msd > calls.html\nt->c/run command\nc->f/read calls.msd\nf->c/text\nc->c/parse, layout, draw^diagram + table + links\nc->t/one HTML page on stdout\nt->f/save calls.html\ne->b/open calls.html\nb->f/load the file^no server, no network\nf->b/self-contained page\nb->e/diagram on top, table below\ne->b/click badge 7 on the diagram\nb->b/jump to row 7^row is highlighted\ne->b/click row 7 back link\nb->b/scroll diagram to event 7\nnote/b/The back link is in the left column.^It stays in view when the table scrolls.\ne->b/switch to side by side^radio switch, no JS\ne->t/edit calls.msd, run callplot again\n".to_string(), MODE_TEXT), 2);
}
