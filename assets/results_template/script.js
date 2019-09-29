$(document).ready(function(){
  // "Fix" 100vh problem on iOS and mobile Chrome
  var wrapper = $("div.results-classic-wrapper");
  window.onresize = function() {
    if ($(window).height() < wrapper.height()) {
      wrapper.height($(window).height());
    } else {
      wrapper.css("height", "");
    }
  };
  window.onresize();

  // Make link to /results/ work as a back button if appropriate
  $("a.js-back-button").on("click", function(e) {
    e.preventDefault();
    if (document.referrer === this.href && window.history.length > 1) {
      window.history.back();
    } else {
      window.location = this.href;
    }
  });

  // Share button functionality
  var timeout;
  $("button#share-button").on("click", function() {
    var share_url = window.location.href;
    if (navigator.share) {
      // use Web Share API if available
      navigator.share({
        url: share_url
      });
    } else {
      // otherwise copy to clipboard
      let dummy = document.createElement('input');
      document.body.append(dummy);
      dummy.value = share_url;
      dummy.select();
      document.execCommand('copy');
      document.body.removeChild(dummy);

      // show snack
      window.clearTimeout(timeout);
      var display_snack = function() {
        $("div#share-snack div.snackbar-body").html("Link copied! " + share_url);
        $("div#share-snack").addClass("show");
        timeout = window.setTimeout(function() {
          $("div#share-snack").removeClass("show");
        }, 2000);
      };
      if ($("div#share-snack").hasClass("show")) {
        $("div#share-snack").removeClass("show");
        window.setTimeout(display_snack, 200);
      } else {
        display_snack();
      }
    }
  });

  // First, make sure all default checkboxes are checked initially (browser
  // might tend to remember previous state and it's not apparent to the user
  // that the boxes would be unchecked without going into the menu)
  $("div#filters input").prop("checked", true);

  // Correct minimum width of header and footnotes based on number of events
  var fix_width = function(extra) {
    let width = $("colgroup.event-columns col").length * 2 + 28 + extra;
    let min_width = width + 0.5;
    $("div.results-classic-thead-background").css("min-width", min_width + "em");
    $("div.results-classic-header").css("width", width + "em");
    $("div.results-classic-footnotes").css("width", width + "em");
  };
  // called later by sort_and_toggle_event_rank(), and intial values set by
  // inline CSS anyways (to prevent jump when JS loads)

  // Highlight table columns on hover
  // Adapted from https://css-tricks.com/row-and-column-highlighting/
  $("table.results-classic td, table.results-classic th").hover(
    function() {
      $("colgroup col").eq($(this).index()).addClass("hover");
    }, function() {
      $("colgroup col").eq($(this).index()).removeClass("hover");
    });

  // Sort teams (rows) by various things
  // Adapted from https://blog.niklasottosson.com/javascript/jquery-sort-table-rows-on-column-value/
  var sort_select = function() {
    let thing = $("#sort-select option:selected").val()

    var sort_by_number = function(a, b) {
      let number_a = parseInt($(a).find("td.number").text());
      let number_b = parseInt($(b).find("td.number").text());

      return number_a - number_b;
    }

    var sort_by_school = function(a, b) {
      let school_a = $(a).find("td.team").text();
      let school_b = $(b).find("td.team").text();

      if (school_a > school_b) { return 1; }
      else if (school_a < school_b) { return -1; }
      else { return sort_by_number(a, b); }
    }

    var sort_by_rank = function(a, b) {
      let rank_col = $("#event-select option:selected").val();

      if (rank_col === "all") {
        var rank_a = parseInt($(a).find("td.rank").text());
        var rank_b = parseInt($(b).find("td.rank").text());
      } else {
        var rank_a = parseInt($(a).find("td.event-points").eq(rank_col).attr("data-place"));
        var rank_b = parseInt($(b).find("td.event-points").eq(rank_col).attr("data-place"));
      }

      let diff = rank_a - rank_b;
      if (diff !== 0) { return diff; }
      else { return sort_by_number(a, b); }
    }

    var sort_by_state = function(a, b) {
      let state_a = $(a).find("td.team small").text();
      let state_b = $(b).find("td.team small").text();

      if (state_a > state_b) { return 1; }
      else if (state_a < state_b) { return -1; }
      else { return sort_by_number(a, b); }
    }

    switch(thing) {
      case "number":
        var sort_fun = sort_by_number;
        break;
      case "school":
        var sort_fun = sort_by_school;
        break;
      case "rank":
        var sort_fun = sort_by_rank;
        break;
      case "state":
        var sort_fun = sort_by_state;
        break;
      default:
        return;
    }

    let rows = $("table.results-classic tbody tr").get();
    rows.sort(sort_fun);

    $.each(rows, function(index, row) {
      $("table.results-classic tbody").append(row);
    });
  }
  sort_select(); // call sort immediately if selection is different from default
  $("#sort-select").change(sort_select); // call sort on selection change

  // Show the hidden column with event points directly next to team name for
  // small screens
  var sort_and_toggle_event_rank = function() {
    let rank_col = $("#event-select option:selected").val();

    // re-sort if by rank is selected (needed because rank would be by event)
    if($("#sort-select option:selected").val() === "rank") {
      sort_select();
    }

    if (rank_col !== "all") {
      $("div.results-classic-wrapper").addClass("event-focused");
      $("th.event-points-focus div").text($("#event-select option:selected").text());

      // copy info from event-points to event-points-focus
      let rows = $("table.results-classic tbody tr").get();
      $.each(rows, function(index, row) {
        let source_elem = $(row).find("td.event-points").eq(rank_col);
        let source_html = source_elem.html();
        let points = source_elem.attr("data-points");
        let points_elem = $(row).find("td.event-points-focus");
        points_elem.children("div").html(source_html);
        points_elem.attr("data-points", points);
      });

      // update medal highlighting based on checkboxes
      $("div#settings input").each(function(index) {
        let medal_number = $(this).attr("id").slice("medal".length);
        let medals = $("td.event-points-focus[data-points='" + medal_number + "'] div")

        if ($(this).prop("checked")) {
          medals.removeAttr("style"); // default state is 'highlighted'
        } else {
          medals.css("background-color", "transparent");
        }
      });
      fix_width(4);

    } else {
      $("div.results-classic-wrapper").removeClass("event-focused");
      $("th.event-points-focus div").text("");
      $("td.event-points-focus div").text("");
      fix_width(0);
    }
  }
  sort_and_toggle_event_rank();
  $("#event-select").change(sort_and_toggle_event_rank);

  // Toggle rows based on checkboxes
  $("div#team-filter input").change(function() {
    let id = $(this).attr("id");
    let all_box = $("div#team-filter input#allTeams");
    let team_boxes = $("div#team-filter input").not("#allTeams");

    if (id === "allTeams") {
      if ($(this).prop("checked")) {
        team_boxes.not(":checked").trigger("click");
      } else {
        team_boxes.filter(":checked").trigger("click");
      }
      all_box.prop("indeterminate", false);

    } else {
      let team_number = id.slice("team".length);
      let r = "table.results-classic tr[data-team-number='" + team_number + "']";

      if ($(this).prop("checked")) {
        $(r).show();
      } else {
        $(r).hide();
      }
      if (team_boxes.not(":checked").length === 0) {
        all_box.prop("indeterminate", false);
        all_box.prop("checked", true);

      } else if (team_boxes.filter(":checked").length === 0) {
        all_box.prop("indeterminate", false);
        all_box.prop("checked", false);

      } else {
        all_box.prop("indeterminate", true);
      }
    }
  });

  // Toggle medal highlighting based on checkboxes
  $("div#medal-filter input").change(function() {
      let medal_number = $(this).attr("id").slice("medal".length);
      let medals = $("td[data-points='" + medal_number + "'] div")
      
      if ($(this).prop("checked")) {
        medals.removeAttr("style"); // default state is 'highlighted'
      } else {
        medals.css("background-color", "transparent");
      }
  });

  // Populate Team Detail table
  $("td.number a").on("click", function() {
    let source_row = $(this).closest("tr");
    let table_rows = $("div#team-detail table tbody").children();
    $.each(source_row.children("td.event-points"), function (index, td) {
      let dest_row = table_rows.eq(index);
      dest_row.children().eq(1).html($(td).attr("data-place"));
      dest_row.children().eq(2).html($(td).attr("data-points"));
      console.log(dest_row);
    });
  });

  // Enable popovers for further explanation of badge abbrevs
  $(function () {
    $('[data-toggle="popover"]').popover()
  })
});
