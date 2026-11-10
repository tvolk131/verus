(* Inspection only: same summary_disk layout and reader as Coq 8.9.1
   library/library.ml:86-90,430-433. This reads already coqchk-checked objects;
   it neither imports them into a proof nor changes any kernel checking. *)
type summary_disk = {
  md_name : Names.DirPath.t;
  md_imports : Names.DirPath.t array;
  md_deps : (Names.DirPath.t * Safe_typing.vodigest) array;
}

let () =
  for i = 1 to Array.length Sys.argv - 1 do
    let file = Sys.argv.(i) in
    let ch = System.with_magic_number_check
      (System.raw_intern_state Coq_config.vo_magic_number) file in
    let (summary : summary_disk), _, _ = System.marshal_in_segment file ch in
    close_in ch;
    let dependencies = Array.to_list summary.md_deps |> List.map (fun (name, digest) ->
      match digest with
      | Safe_typing.Dvo_or_vi d -> Names.DirPath.to_string name ^ "=" ^ Digest.to_hex d
      | Safe_typing.Dvivo _ -> failwith "unexpected vio-derived dependency") in
    Printf.printf "%s\t%s\t%s\t%s\n" (Names.DirPath.to_string summary.md_name) file
      (String.concat ";" dependencies)
      (String.concat ";" (Array.to_list summary.md_imports |> List.map Names.DirPath.to_string))
  done
