jsxs(Fragment, {
    children: [
        jsxs("style", {
            children: [
                ".jsx-styled-8b210918{",
                "background-color: #2e3440ff",
                "}",
                ".jsx-styled-21f603ec{",
                "color: #81A1C1",
                "}",
                ".jsx-styled-221f03f4{",
                "color: #88C0D0",
                "}",
                ".jsx-styled-22720413{",
                "color: #A3BE8C",
                "}",
                ".jsx-styled-22b50420{",
                "color: #D8DEE9",
                "}",
                ".jsx-styled-2bc704ac{",
                "color: #D8DEE9FF",
                "}",
                ".jsx-styled-22f6042a{",
                "color: #ECEFF4",
                "}"
            ]
        }),
        jsx("pre", {
            tabIndex: "0",
            className: "shiki nord jsx-styled-8b210918",
            "children": jsx(Trans, {
                "id": "7qnlqmoTBAR56MjNefjV9EalHBjIiyw3nT0Y6Dc6LGc=",
                "message": "<code><span><span2>const</span2><span3> </span3><span4>fs</span4><span5> </span5><span6>=</span6><span7> </span7><span8>require</span8><span9>(</span9><span10>'</span10><span11>fs</span11><span12>'</span12><span13>)</span13></span>{LF}<span14><span15>const</span15><span16> </span16><span17>markdown</span17><span18> </span18><span19>=</span19><span20> </span20><span21>require</span21><span22>(</span22><span23>'</span23><span24>markdown-it</span24><span25>'</span25><span26>)</span26></span14>{LF}<span27><span28>const</span28><span29> </span29><span30>shiki</span30><span31> </span31><span32>=</span32><span33> </span33><span34>require</span34><span35>(</span35><span36>'</span36><span37>shiki</span37><span38>'</span38><span39>)</span39></span27>{LF}<span40></span40>{LF}<span41><span42>shiki</span42><span43>.</span43><span44>getHighlighter</span44><span45>(</span45><span46>{LC}</span46></span41>{LF}<span47><span48>  </span48><span49>theme</span49><span50>:</span50><span51> </span51><span52>'</span52><span53>nord</span53><span54>'</span54></span47>{LF}<span55><span56>{RC}</span56><span57>)</span57><span58>.</span58><span59>then</span59><span60>(</span60><span61>highlighter</span61><span62> </span62><span63>={GT}</span63><span64> </span64><span65>{LC}</span65></span55>{LF}<span66><span67>  </span67><span68>const</span68><span69> </span69><span70>md</span70><span71> </span71><span72>=</span72><span73> </span73><span74>markdown</span74><span75>(</span75><span76>{LC}</span76></span66>{LF}<span77><span78>    </span78><span79>html</span79><span80>:</span80><span81> </span81><span82>true</span82><span83>,</span83></span77>{LF}<span84><span85>    </span85><span86>highlight</span86><span87>:</span87><span88> </span88><span89>(</span89><span90>code</span90><span91>,</span91><span92> </span92><span93>lang</span93><span94>)</span94><span95> </span95><span96>={GT}</span96><span97> </span97><span98>{LC}</span98></span84>{LF}<span99><span100>      </span100><span101>return</span101><span102> </span102><span103>highlighter</span103><span104>.</span104><span105>codeToHtml</span105><span106>(</span106><span107>code</span107><span108>,</span108><span109> </span109><span110>{LC}</span110><span111> </span111><span112>lang</span112><span113> </span113><span114>{RC}</span114><span115>)</span115></span99>{LF}<span116><span117>    </span117><span118>{RC}</span118></span116>{LF}<span119><span120>  </span120><span121>{RC}</span121><span122>)</span122></span119>{LF}<span123></span123>{LF}<span124><span125>  </span125><span126>const</span126><span127> </span127><span128>html</span128><span129> </span129><span130>=</span130><span131> </span131><span132>md</span132><span133>.</span133><span134>render</span134><span135>(</span135><span136>fs</span136><span137>.</span137><span138>readFileSync</span138><span139>(</span139><span140>'</span140><span141>index.md</span141><span142>'</span142><span143>,</span143><span144> </span144><span145>'</span145><span146>utf-8</span146><span147>'</span147><span148>))</span148></span124>{LF}<span149><span150>  </span150><span151>const</span151><span152> </span152><span153>out</span153><span154> </span154><span155>=</span155><span156> </span156><span157>`</span157></span149>{LF}<span158><span159>    {LT}title{GT}Shiki{LT}/title{GT}</span159></span158>{LF}<span160><span161>    {LT}link rel=\"stylesheet\" href=\"style.css\"{GT}</span161></span160>{LF}<span162><span163>    </span163><span164>${LC}</span164><span165>html</span165><span166>{RC}</span166></span162>{LF}<span167><span168>    {LT}script src=\"index.js\"{GT}{LT}/script{GT}</span168></span167>{LF}<span169><span170>  </span170><span171>`</span171></span169>{LF}<span172><span173>  </span173><span174>fs</span174><span175>.</span175><span176>writeFileSync</span176><span177>(</span177><span178>'</span178><span179>index.html</span179><span180>'</span180><span181>,</span181><span182> </span182><span183>out</span183><span184>)</span184></span172>{LF}<span185></span185>{LF}<span186><span187>  </span187><span188>console</span188><span189>.</span189><span190>log</span190><span191>(</span191><span192>'</span192><span193>done</span193><span194>'</span194><span195>)</span195></span186>{LF}<span196><span197>{RC}</span197><span198>)</span198></span196>{LF}<span199></span199></code>",
                "components": {
                    "code": jsxs("code", {}),
                    "span": jsxs("span", {
                        className: "line"
                    }),
                    "span2": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span3": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span4": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span5": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span6": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span7": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span8": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span9": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span10": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span11": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span12": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span13": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span14": jsxs("span", {
                        className: "line"
                    }),
                    "span15": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span16": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span17": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span18": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span19": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span20": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span21": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span22": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span23": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span24": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span25": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span26": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span27": jsxs("span", {
                        className: "line"
                    }),
                    "span28": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span29": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span30": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span31": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span32": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span33": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span34": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span35": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span36": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span37": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span38": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span39": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span40": jsx("span", {
                        className: "line"
                    }),
                    "span41": jsxs("span", {
                        className: "line"
                    }),
                    "span42": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span43": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span44": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span45": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span46": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span47": jsxs("span", {
                        className: "line"
                    }),
                    "span48": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span49": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span50": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span51": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span52": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span53": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span54": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span55": jsxs("span", {
                        className: "line"
                    }),
                    "span56": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span57": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span58": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span59": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span60": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span61": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span62": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span63": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span64": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span65": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span66": jsxs("span", {
                        className: "line"
                    }),
                    "span67": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span68": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span69": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span70": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span71": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span72": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span73": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span74": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span75": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span76": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span77": jsxs("span", {
                        className: "line"
                    }),
                    "span78": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span79": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span80": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span81": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span82": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span83": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span84": jsxs("span", {
                        className: "line"
                    }),
                    "span85": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span86": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span87": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span88": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span89": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span90": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span91": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span92": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span93": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span94": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span95": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span96": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span97": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span98": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span99": jsxs("span", {
                        className: "line"
                    }),
                    "span100": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span101": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span102": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span103": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span104": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span105": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span106": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span107": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span108": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span109": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span110": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span111": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span112": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span113": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span114": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span115": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span116": jsxs("span", {
                        className: "line"
                    }),
                    "span117": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span118": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span119": jsxs("span", {
                        className: "line"
                    }),
                    "span120": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span121": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span122": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span123": jsx("span", {
                        className: "line"
                    }),
                    "span124": jsxs("span", {
                        className: "line"
                    }),
                    "span125": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span126": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span127": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span128": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span129": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span130": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span131": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span132": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span133": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span134": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span135": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span136": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span137": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span138": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span139": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span140": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span141": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span142": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span143": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span144": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span145": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span146": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span147": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span148": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span149": jsxs("span", {
                        className: "line"
                    }),
                    "span150": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span151": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span152": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span153": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span154": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span155": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span156": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span157": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span158": jsx("span", {
                        className: "line"
                    }),
                    "span159": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span160": jsx("span", {
                        className: "line"
                    }),
                    "span161": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span162": jsxs("span", {
                        className: "line"
                    }),
                    "span163": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span164": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span165": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span166": jsx("span", {
                        className: "jsx-styled-21f603ec"
                    }),
                    "span167": jsx("span", {
                        className: "line"
                    }),
                    "span168": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span169": jsxs("span", {
                        className: "line"
                    }),
                    "span170": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span171": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span172": jsxs("span", {
                        className: "line"
                    }),
                    "span173": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span174": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span175": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span176": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span177": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span178": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span179": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span180": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span181": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span182": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span183": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span184": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span185": jsx("span", {
                        className: "line"
                    }),
                    "span186": jsxs("span", {
                        className: "line"
                    }),
                    "span187": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span188": jsx("span", {
                        className: "jsx-styled-22b50420"
                    }),
                    "span189": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span190": jsx("span", {
                        className: "jsx-styled-221f03f4"
                    }),
                    "span191": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span192": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span193": jsx("span", {
                        className: "jsx-styled-22720413"
                    }),
                    "span194": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span195": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span196": jsxs("span", {
                        className: "line"
                    }),
                    "span197": jsx("span", {
                        className: "jsx-styled-22f6042a"
                    }),
                    "span198": jsx("span", {
                        className: "jsx-styled-2bc704ac"
                    }),
                    "span199": jsx("span", {
                        className: "line"
                    })
                },
                "values": {
                    "LF": jsx("br", {}),
                    "LT": jsx(Fragment, {
                        "children": "<"
                    }),
                    "GT": jsx(Fragment, {
                        "children": ">"
                    }),
                    "LC": jsx(Fragment, {
                        "children": "{"
                    }),
                    "RC": jsx(Fragment, {
                        "children": "}"
                    })
                }
            })
        }),
        "\n"
    ]
});